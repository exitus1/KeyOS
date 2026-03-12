#![no_std]

use core::{fmt::Write, num::NonZeroUsize};

use num_traits::FromPrimitive;
use xous::MessageId;
use xous_api_log::api;

const RINGBUFFER_SIZE: usize = 16 * 1024;
const LOG_RECORD_TERMINATOR: u8 = 0x1e;

macro_rules! log {
    ($server:expr, $($arg:tt)*) => {{
        $server.ring.log_internal(format_args!($($arg)*));
    }};
}

struct LogServer {
    ring: RingStore,
    readers: [LogReader; 4],
    last_panic_message: PanicMessage,
}

struct RingStore {
    ringbuffer: [u8; RINGBUFFER_SIZE],
    write_offset: usize,
    buffer_filled: bool,
}

struct LogReader {
    pid: Option<xous::PID>,
    read_offset: usize,
    read_msg: Option<xous::MessageEnvelope>,
}

struct PanicMessage {
    pid: Option<xous::PID>,
    buf: [u8; api::PANIC_MESSAGE_SIZE],
    len: usize,
    in_progress: bool,
}

impl PanicMessage {
    const fn new() -> Self {
        PanicMessage { pid: None, buf: [0; api::PANIC_MESSAGE_SIZE], len: 0, in_progress: false }
    }

    fn read(&self, out: &mut [u8]) -> usize {
        let Some(pid) = self.pid else {
            return 0;
        };

        // Write PID followed by the panic message bytes
        if out.is_empty() {
            return 0;
        }

        let len = self.len.min(out.len() - 1);
        out[0] = pid.get();
        out[1..len + 1].copy_from_slice(&self.buf[..len]);

        len + 1
    }

    fn write(&mut self, b: &[u8]) {
        let len = b.len().min(self.buf.len() - self.len);
        self.buf[self.len..self.len + len].copy_from_slice(&b[..len]);
        self.len = self.len.saturating_add(len);
    }

    fn begin(&mut self, pid: xous::PID) {
        self.clear();
        self.pid = Some(pid);
        self.in_progress = true;
    }

    fn finish(&mut self) { self.in_progress = false; }

    fn body(&self) -> &[u8] { &self.buf[..self.len] }

    fn pid(&self) -> Option<xous::PID> { self.pid }

    fn clear(&mut self) {
        self.pid = None;
        self.len = 0;
        self.in_progress = false;
        self.buf.fill(0);
    }
}

const EMPTY_LOG_READER: LogReader = LogReader { pid: None, read_offset: 0, read_msg: None };

static mut SERVER: LogServer = LogServer {
    ring: RingStore { ringbuffer: [0; RINGBUFFER_SIZE], write_offset: 0, buffer_filled: false },
    readers: [EMPTY_LOG_READER, EMPTY_LOG_READER, EMPTY_LOG_READER, EMPTY_LOG_READER],
    last_panic_message: PanicMessage::new(),
};

impl RingStore {
    fn write_bytes(&mut self, b: &[u8]) {
        let len = b.len().min(RINGBUFFER_SIZE);

        // Part 1: from current cursor to end
        //        [         |--->]
        let part1 = len.min(RINGBUFFER_SIZE - self.write_offset);
        self.ringbuffer[self.write_offset..self.write_offset + part1].copy_from_slice(&b[..part1]);

        // Part 2: from beginning
        //        [---->    |    ]
        let part2 = len - part1;
        if part2 > 0 {
            self.ringbuffer[..part2].copy_from_slice(&b[part1..part1 + part2]);
        }
        // Note: we might overtake the readers' read_offset here, but that means that they
        //       are way too slow, so we will be losing logs anyway, we might as well lose
        //       a full ringbuffer's worth.
        self.write_offset += len;
        if self.write_offset > RINGBUFFER_SIZE {
            self.buffer_filled = true;
            self.write_offset %= RINGBUFFER_SIZE;
        }
    }

    fn write_terminated(&mut self, payload: &[u8]) {
        self.write_bytes(payload);
        self.write_bytes(&[LOG_RECORD_TERMINATOR]);
    }

    fn log_internal(&mut self, args: core::fmt::Arguments<'_>) {
        core::fmt::write(self, args).ok();
        self.write_bytes(&[LOG_RECORD_TERMINATOR]);
    }
}

impl Write for RingStore {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_bytes(s.as_bytes());
        Ok(())
    }
}

impl LogServer {
    fn handle_message(&mut self, opcode: api::Opcode, mut envelope: xous::MessageEnvelope) {
        use api::Opcode::*;
        match opcode {
            StandardOutput | StandardError => {
                // temp fix: flush pending panic output when the first regular log
                // message arrives. The intended behavior is to flush only on PanicFinished,
                // but our std panic writer currently does not emit that reliably.
                // https://github.com/Foundation-Devices/rust-keyos/blob/1.91.1-xous-arm/library/std/src/sys/stdio/xous.rs#L116C1-L125C2
                self.flush_pending_panic(false);
                let Some(mem) = envelope.body.memory_message() else {
                    return;
                };
                let len = mem.valid.map(|v| v.get()).unwrap_or_default().min(mem.buf.len());
                self.ring.write_terminated(&mem.buf.as_slice()[..len]);
            }
            ProgramName => {}
            PanicStarted => {
                self.flush_pending_panic(false);

                let pid = envelope.sender.pid().unwrap();
                self.last_panic_message.begin(pid);
            }
            PanicMessage0 => {}
            PanicMessage1 | PanicMessage2 | PanicMessage3 | PanicMessage4 | PanicMessage5 | PanicMessage6
            | PanicMessage7 | PanicMessage8 | PanicMessage9 | PanicMessage10 | PanicMessage11
            | PanicMessage12 | PanicMessage13 | PanicMessage14 | PanicMessage15 | PanicMessage16
            | PanicMessage17 | PanicMessage18 | PanicMessage19 | PanicMessage20 | PanicMessage21
            | PanicMessage22 | PanicMessage23 | PanicMessage24 | PanicMessage25 | PanicMessage26
            | PanicMessage27 | PanicMessage28 | PanicMessage29 | PanicMessage30 | PanicMessage31
            | PanicMessage32 => {
                if !self.last_panic_message.in_progress {
                    return;
                }

                let Some(scalar) = envelope.body.scalar_message() else {
                    return;
                };
                let mut output_bfr = [0u8; core::mem::size_of::<usize>() * 4];
                let output_iter = output_bfr.iter_mut();

                // Combine the four arguments to form a single
                // contiguous buffer. Note: The buffer size will change
                // depending on the platform's `usize` length.
                let arg1_bytes = scalar.arg1.to_le_bytes();
                let arg2_bytes = scalar.arg2.to_le_bytes();
                let arg3_bytes = scalar.arg3.to_le_bytes();
                let arg4_bytes = scalar.arg4.to_le_bytes();
                let input_iter = arg1_bytes
                    .iter()
                    .chain(arg2_bytes.iter())
                    .chain(arg3_bytes.iter())
                    .chain(arg4_bytes.iter());
                for (dest, src) in output_iter.zip(input_iter) {
                    *dest = *src;
                }
                let total_chars = opcode as usize - PanicMessage0 as usize;
                let buf = &output_bfr[..total_chars];
                self.last_panic_message.write(buf);
            }
            PanicFinished => {
                self.flush_pending_panic(true);
            }
            ReadLogs => {
                let Some(pid) = envelope.sender.pid() else {
                    return;
                };
                for reader in &mut self.readers {
                    if reader.pid.is_none() {
                        // We reached the end, allocate into this slot
                        reader.pid = Some(pid);
                        if self.ring.buffer_filled {
                            // We already filled the ringbuffer, send the whole thing:
                            // [---->WR----]
                            reader.read_offset = (self.ring.write_offset + 1) % RINGBUFFER_SIZE;
                        } else {
                            // We are still filling the buffer:
                            // [R---->W    ]
                            reader.read_offset = 0;
                        }
                    }
                    if reader.pid == Some(pid) {
                        // If read_msg is already filled (very unlikely), dropping it is OK, it will just
                        // return the blocking call.
                        reader.read_msg = Some(envelope);
                        return;
                    }
                }
            }

            ReadLastPanicMessage => {
                let Some(mem) = envelope.body.memory_message_mut() else {
                    return;
                };

                let mut len = self.last_panic_message.read(mem.buf.as_slice_mut());
                let panic_pid = self.last_panic_message.pid();
                self.last_panic_message.clear();

                // Append backtrace from kernel's panic message (if present and PID matches)
                if len < mem.buf.len() {
                    let mut panic_from_kernel_buf = [0u8; 512];
                    let kernel_range = unsafe {
                        xous::MemoryRange::new(
                            panic_from_kernel_buf.as_mut_ptr() as usize,
                            panic_from_kernel_buf.len(),
                        )
                    };
                    if let Ok(range) = kernel_range {
                        if let Ok((panic_pid_from_kernel, panic_from_kernel_len)) =
                            xous::get_panic_message(range)
                        {
                            // Only append if the kernel panic message is for the same process
                            if Some(panic_pid_from_kernel) == panic_pid.map(|p| p.get()) {
                                // Find "\nBacktrace:" marker to extract only the backtrace
                                let kernel_msg = &panic_from_kernel_buf[..panic_from_kernel_len];
                                let backtrace_start = kernel_msg
                                    .windows(11)
                                    .position(|w| w == b"\nBacktrace:")
                                    .unwrap_or(panic_from_kernel_len);
                                let backtrace = &kernel_msg[backtrace_start..];
                                let copy_len = backtrace.len().min(mem.buf.len() - len);
                                mem.buf.as_slice_mut::<u8>()[len..len + copy_len]
                                    .copy_from_slice(&backtrace[..copy_len]);
                                len += copy_len;
                            }
                        }
                    }
                }

                mem.valid = if len > 0 { Some(NonZeroUsize::new(len).unwrap()) } else { None };
                mem.offset = None;
            }
        };
    }

    fn emit_panic_frame(&mut self) {
        let Some(pid) = self.last_panic_message.pid() else {
            return;
        };
        let body = self.last_panic_message.body();
        core::fmt::write(&mut self.ring, format_args!("PANIC in PID {}: ", pid.get())).ok();
        self.ring.write_bytes(body);
        self.ring.write_bytes(&[LOG_RECORD_TERMINATOR]);
    }

    fn flush_pending_panic(&mut self, force_finish: bool) {
        if !self.last_panic_message.in_progress {
            return;
        }

        if self.last_panic_message.body().is_empty() {
            if force_finish {
                self.last_panic_message.finish();
            }
            return;
        }

        self.emit_panic_frame();
        self.last_panic_message.finish();
    }

    fn send_logs(&mut self) {
        for reader in &mut self.readers {
            if reader.read_offset == self.ring.write_offset {
                continue;
            }
            let Some(mut envelope) = reader.read_msg.take() else {
                continue;
            };
            let Some(mem) = envelope.body.memory_message_mut() else {
                continue;
            };

            // Cases with big enough message buffer:
            // [   R--->W   ]
            // [->W     R---]

            // Cases with small message buffer:
            // [   R--->  W ]
            // [  W R--->   ]
            // [->  W   R---]

            // Part 1: from current read cursor to end or write cursor
            let part1_end = if reader.read_offset <= self.ring.write_offset {
                self.ring.write_offset
            } else {
                RINGBUFFER_SIZE
            };
            let part1_len = (part1_end - reader.read_offset).min(mem.buf.len());
            mem.buf.as_slice_mut()[..part1_len]
                .copy_from_slice(&self.ring.ringbuffer[reader.read_offset..reader.read_offset + part1_len]);
            mem.valid = NonZeroUsize::new(part1_len);
            reader.read_offset = (reader.read_offset + part1_len) % RINGBUFFER_SIZE;

            // Part 2: from beginning to write cursor
            if reader.read_offset == 0 {
                let part2_len = self.ring.write_offset.min(mem.buf.len() - part1_len);
                mem.buf.as_slice_mut()[part1_len..part1_len + part2_len]
                    .copy_from_slice(&self.ring.ringbuffer[..part2_len]);
                mem.valid = NonZeroUsize::new(part1_len + part2_len);
                reader.read_offset = part2_len;
            }
        }
    }

    fn run(&mut self) -> ! {
        xous::set_thread_priority(xous::ThreadPriority::System8).unwrap();
        log!(self, "[LOG] Starting with PID {}", xous::process::id());
        // TODO: Only allow privileged clients to read logs (SFT-5025)
        let server_addr = xous::create_server_with_sid(
            xous::SID::from_bytes(b"xous-log-server ").unwrap(),
            0..MessageId::MAX,
        )
        .expect("create server");
        log!(self, "[LOG] Server listening on address {:?}", server_addr);

        let mut counter: usize = 0;
        loop {
            if counter.trailing_zeros() >= 12 {
                log!(self, "[LOG] Counter tick: {}", counter);
            }
            counter += 1;
            let envelope = xous::syscall::receive_message(server_addr).expect("couldn't get address");
            if let Some(opcode) = FromPrimitive::from_usize(envelope.body.id()) {
                self.handle_message(opcode, envelope);
            } else {
                log!(
                    self,
                    "[LOG] Unrecognized opcode from process {}: {}",
                    envelope.sender.pid().map(|v| v.get()).unwrap_or_default(),
                    envelope.body.id()
                );
            }
            self.send_logs();
        }
    }
}

fn main() -> ! { unsafe { (&mut *core::ptr::addr_of_mut!(SERVER)).run() } }
