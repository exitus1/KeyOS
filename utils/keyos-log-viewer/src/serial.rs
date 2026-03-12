// SPDX-FileCopyrightText: 2026 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Read;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::thread;
use std::time::Duration;

use serialport::SerialPort;

use crate::{AppEvent, SerialEvent};

const LOG_RECORD_TERMINATOR: u8 = 0x1e;

#[derive(Debug, Clone, Copy)]
pub enum SerialCommand {
    RequestProcessList,
}

pub fn start_serial_thread(
    port: String,
    baud: u32,
    reconnect_timeout: Duration,
    sender: Sender<AppEvent>,
    command_receiver: Receiver<SerialCommand>,
) {
    thread::spawn(move || {
        let mut state = SerialState::new(port, baud, reconnect_timeout, sender, command_receiver);
        state.run();
    });
}

struct SerialState {
    port: String,
    baud: u32,
    reconnect_timeout: Duration,
    sender: Sender<AppEvent>,
    command_receiver: Receiver<SerialCommand>,
    buffer: Vec<u8>,
    last_status: String,
}

#[derive(Debug)]
enum SerialThreadError {
    ChannelClosed,
    SerialWriteFailed,
}

impl SerialState {
    fn new(
        port: String,
        baud: u32,
        reconnect_timeout: Duration,
        sender: Sender<AppEvent>,
        command_receiver: Receiver<SerialCommand>,
    ) -> Self {
        Self {
            port,
            baud,
            reconnect_timeout,
            sender,
            command_receiver,
            buffer: Vec::new(),
            last_status: String::new(),
        }
    }

    fn run(&mut self) { let _ = self.run_inner(); }

    fn run_inner(&mut self) -> Result<(), SerialThreadError> {
        self.send_status("Initializing...")?;

        loop {
            let mut serial_port = self.connect_serial()?;

            self.buffer.clear();

            self.process_commands(&mut *serial_port)?;

            let mut read_buf = [0; 1024];
            loop {
                self.process_commands(&mut *serial_port)?;

                match serial_port.read(&mut read_buf) {
                    Ok(n) if n > 0 => {
                        self.buffer.extend_from_slice(&read_buf[..n]);
                        self.process_received_bytes()?;
                    }
                    Ok(_) => {}
                    Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => continue,
                    Err(e) => {
                        self.send_status(&format!("Serial error: {}", e))?;
                        break;
                    }
                }
            }
        }
    }

    fn send_status(&mut self, status: &str) -> Result<(), SerialThreadError> {
        if self.last_status == status {
            return Ok(());
        }

        self.last_status = status.to_string();
        self.sender
            .send(AppEvent::Serial(SerialEvent::Status(status.to_string())))
            .map_err(|_| SerialThreadError::ChannelClosed)
    }

    fn connect_serial(&mut self) -> Result<Box<dyn SerialPort>, SerialThreadError> {
        let mut reconnect_count = 0u64;

        loop {
            match serialport::new(&self.port, self.baud).timeout(Duration::from_secs(1)).open() {
                Ok(serial_port) => {
                    self.send_status("Connected")?;
                    return Ok(serial_port);
                }
                Err(err) => {
                    reconnect_count += 1;
                    let elapsed = reconnect_count;
                    if elapsed < self.reconnect_timeout.as_secs() {
                        self.send_status(&format!(
                            "Disconnected - Reconnecting in {} seconds... ({})",
                            self.reconnect_timeout.as_secs() - elapsed,
                            err
                        ))?;
                        thread::sleep(Duration::from_secs(1));
                    } else {
                        self.send_status(&format!("Failed to connect to {} ({})", self.port, err))?;
                        thread::sleep(self.reconnect_timeout);
                        reconnect_count = 0;
                    }
                }
            }
        }
    }

    fn process_commands(&mut self, serial_port: &mut dyn SerialPort) -> Result<(), SerialThreadError> {
        loop {
            match self.command_receiver.try_recv() {
                Ok(SerialCommand::RequestProcessList) => {
                    serial_port.write(b"t").map_err(|err| {
                        let _ = self.send_status(&format!("Failed to send process-list request: {}", err));
                        SerialThreadError::SerialWriteFailed
                    })?;
                }
                Err(TryRecvError::Empty) => return Ok(()),
                Err(TryRecvError::Disconnected) => return Err(SerialThreadError::ChannelClosed),
            }
        }
    }

    fn process_received_bytes(&mut self) -> Result<(), SerialThreadError> {
        while let Some(pos) = self.buffer.iter().position(|b| *b == LOG_RECORD_TERMINATOR) {
            let payload =
                String::from_utf8_lossy(&self.buffer[..pos]).trim_end_matches(['\r', '\n']).to_string();
            self.buffer.drain(..=pos);

            if payload.is_empty() {
                continue;
            }

            self.sender
                .send(AppEvent::Serial(SerialEvent::Payload(payload)))
                .map_err(|_| SerialThreadError::ChannelClosed)?;
        }

        Ok(())
    }
}
