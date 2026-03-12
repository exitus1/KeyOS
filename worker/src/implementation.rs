// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
    collections::{binary_heap::PeekMut, BinaryHeap, LinkedList},
    sync::mpsc,
    time::{Duration, Instant},
};

use server::{xous, xous_names, AllPermissions, MessageId as _, ServerContext};
use xous_ticktimer::TicktimerCallback;

const MAX_SUBS: usize = 32;
const REQUESTS_START: usize = MAX_SUBS * 2;
const MAX_REQUESTS: usize = 16;
const TOTAL_MESSAGES: usize = REQUESTS_START + MAX_REQUESTS;

macro_rules! test_log {
    ($($arg:tt)*) => {
        #[cfg(feature = "integration_test")]
        log::debug!($($arg)*);
    };
}

#[derive(Debug, Clone)]
pub struct WorkerApiInner {
    pub conn: server::CheckedConn<AllPermissions>,
    pub tx: mpsc::Sender<WorkerEvent>,
}

impl Drop for WorkerApiInner {
    fn drop(&mut self) { self.conn.try_send_scalar(Shutdown).ok(); }
}

impl WorkerApiInner {
    #[inline]
    pub(crate) fn queue_event(&self, event: WorkerEvent) {
        if self.tx.send(event).is_ok() {
            self.conn.try_send_scalar(Wake).ok();
        }
    }
}

impl Default for WorkerApiInner {
    fn default() -> Self {
        let pid = xous::current_pid().unwrap();
        let (tx, rx) = mpsc::channel();
        let sid = server::create_sid("");

        // by default allow all messages
        xous::allow_messages_on_server(sid, 0..TOTAL_MESSAGES).unwrap();

        let mut server = WorkerServer {
            sid,
            rx,
            names: xous_names::XousNames::new().unwrap(),
            connections: Default::default(),

            requests: [const { None }; MAX_REQUESTS],
            subscriptions: [const { None }; MAX_SUBS],

            timers: BinaryHeap::new(),
            retry_cb_active: false,
            retry_queue: LinkedList::new(),
            ticktimer_cb: None,
        };
        let conn = {
            std::thread::spawn(move || server.run());
            let cid = xous::connect_for_process(pid, sid).unwrap();
            cid.into()
        };
        Self { conn, tx }
    }
}

macro_rules! worker_msg {
    ($name:ident, $msg_id:literal) => {
        pub struct $name;
        impl server::AsScalar<1> for $name {
            fn as_scalar(&self) -> [u32; 1] { [0] }
        }
        impl server::FromScalar<1> for $name {
            fn from_scalar(_value: [u32; 1]) -> Self { Self }
        }
        impl server::MessageId for $name {
            const ID: xous::MessageId = $msg_id;
            const SERVER: &'static str = "";
        }
        impl server::Scalar for $name {}
    };
}

worker_msg!(Shutdown, 200);
worker_msg!(Wake, 201);
worker_msg!(RetryCallback, 202);
worker_msg!(TimerCallback, 203);

pub struct WorkerServer {
    sid: xous::SID,
    names: xous_names::XousNames,

    rx: mpsc::Receiver<WorkerEvent>,
    connections: Vec<Cx>,

    requests: [Option<PendingRequest>; MAX_REQUESTS],
    subscriptions: [Option<ActiveSubscription>; MAX_SUBS],

    timers: BinaryHeap<Timer>,
    // linked list for better memory reclamation
    // this should happen relatively infrequently
    retry_queue: LinkedList<HandlerInit>,
    retry_cb_active: bool,
    ticktimer_cb: Option<TicktimerCallback>,
}

pub enum WorkerEvent {
    Task { task: async_task::Runnable },
    Register { init: HandlerInit },
    Timer { timer: Timer },
}

// will retry if result is none
type HandlerInit = Box<dyn FnMut(&mut WorkerServer) -> Option<()> + Send>;

#[derive(Debug)]
pub struct ActiveSubscription {
    pub tx: async_channel::Sender<xous::MessageEnvelope>,
    pub pid: xous::PID,
}

#[derive(Debug)]
pub struct PendingRequest {
    pub tx: Option<oneshot::Sender<Result<xous::MessageEnvelope, xous::Error>>>,
    pub pid: xous::PID,
}

#[derive(Debug)]
pub struct Timer {
    pub instant: Instant,
    pub waker: std::task::Waker,
}

impl PartialEq for Timer {
    fn eq(&self, other: &Self) -> bool { &self.instant == &other.instant }
}

impl Eq for Timer {}

impl PartialOrd for Timer {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> { Some(self.cmp(&other)) }
}

impl Ord for Timer {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering { other.instant.cmp(&self.instant) }
}

#[derive(Debug)]
pub struct Cx {
    pub server_name: &'static str,
    pub cid: xous::CID,
}

impl Drop for Cx {
    fn drop(&mut self) { xous::disconnect(self.cid).ok(); }
}

impl WorkerServer {
    pub fn insert_request(&mut self, msg_id: usize, req: PendingRequest) {
        self.requests[msg_id - REQUESTS_START] = Some(req)
    }

    pub fn insert_subscription(&mut self, msg_id: usize, req: ActiveSubscription) {
        self.subscriptions[msg_id] = Some(req)
    }

    pub fn get_available_request_id(&self) -> Option<usize> {
        let position = self.requests.iter().position(|h| matches!(h, None))?;
        Some(REQUESTS_START + position)
    }

    pub fn get_available_subscription_ids(&self) -> Option<(usize, usize)> {
        let position = self.subscriptions.iter().position(|h| matches!(h, None))?;
        let ids = (position, position + MAX_SUBS);
        Some(ids)
    }

    pub fn try_get_connection_from_name(
        &mut self,
        server_name: &'static str,
    ) -> Result<Option<(xous::CID, xous::CID, xous::PID)>, xous::Error> {
        let slot = self.connections.iter().find(|cx| cx.server_name == server_name);
        let cid = match slot {
            Some(cx) => {
                log::debug!("found active connection for {server_name}");
                cx.cid
            }
            None => {
                log::debug!("no active connection for {server_name}, attempting to create new one");
                let Ok(cid) = self.names.request_connection(server_name) else { return Ok(None) };
                self.connections.push(Cx { server_name, cid });
                cid
            }
        };

        // re-retrieving the PID every time
        // to make sure the PID is still alive
        let pid = xous::get_remote_pid(cid)?;
        let cid_remote = xous::connect_for_process(pid, self.sid)?;

        Ok(Some((cid, cid_remote, pid)))
    }
}

impl WorkerServer {
    fn run(&mut self) {
        let mut context = ServerContext::<Self>::from_raw_sid(self.sid);
        while !context.shutdown {
            let msg = xous::receive_message(self.sid).unwrap();
            let cx = &mut context;
            let msg_id: xous::MessageId = msg.id();
            let pid = msg.sender.pid().unwrap();
            match msg_id {
                // subscription event
                0..MAX_SUBS => {
                    let slot = &mut self.subscriptions[msg_id];
                    match slot {
                        Some(handler) => {
                            if handler.pid == pid {
                                let _res = handler.tx.send_blocking(msg);
                                test_log!("received sub event {_res:?}");
                            } else {
                                log::warn!("received event from invalid PID {pid}, expected {}", handler.pid);
                            }
                        }
                        None => {
                            test_log!("received subscription message, but no handler found");
                        }
                    }
                }
                // subscription cancellation
                MAX_SUBS..REQUESTS_START => {
                    let index = msg_id - MAX_SUBS;
                    let slot = &mut self.subscriptions[index];
                    if slot.as_ref().map_or(false, |sub| sub.pid == pid) {
                        let _slot: Option<ActiveSubscription> = slot.take();
                        test_log!("cancelled sub {index} {msg_id} {}", slot.is_some());
                    }
                }
                // async message responses
                REQUESTS_START..TOTAL_MESSAGES => {
                    let index = msg_id - REQUESTS_START;
                    let slot = &mut self.requests[index];
                    if let Some(handler) = slot {
                        if handler.pid == pid {
                            if let Some(tx) = handler.tx.take() {
                                let _ = tx.send(Ok(msg));
                            }

                            let _slot: Option<PendingRequest> = std::mem::take(slot);
                            test_log!("received response {msg_id} {index} {}", _slot.is_some());
                        } else {
                            log::warn!("received message from invalid PID {pid}, expected {}", handler.pid);
                        }
                    }
                }

                // custom message section
                // in same match so we get "unreachable pattern" warnings
                // in case of message id overlap
                Shutdown::ID => server::handle_scalar_message::<Shutdown, _>(self, msg, cx),
                Wake::ID => server::handle_scalar_message::<Wake, _>(self, msg, cx),
                RetryCallback::ID => server::handle_scalar_message::<RetryCallback, _>(self, msg, cx),
                TimerCallback::ID => server::handle_scalar_message::<TimerCallback, _>(self, msg, cx),
                #[cfg(feature = "integration_test")]
                GetRetryTimerActive::ID => {
                    server::handle_blocking_scalar_message::<GetRetryTimerActive, _>(self, msg, cx)
                }
                _ => {
                    log::warn!("received spurious message id {msg_id} from PID {pid}");
                }
            };
        }
    }

    fn request_retry_cb(&mut self) {
        if !self.retry_cb_active {
            self.request_callback(200, RetryCallback::ID);
            self.retry_cb_active = true;
        }
    }

    fn process_timers(&mut self) {
        const MIN_TIMER: Duration = Duration::from_millis(1);

        let now = Instant::now();
        let duration = loop {
            let Some(timer) = self.timers.peek_mut() else {
                return;
            };
            let duration = timer.instant.checked_duration_since(now).unwrap_or_default();
            if duration < MIN_TIMER {
                let timer = PeekMut::pop(timer);
                timer.waker.wake();
            } else {
                break duration;
            }
        };

        test_log!("sleeping timer for {duration:?} {}", self.timers.len());
        self.request_callback(duration.as_millis() as usize, TimerCallback::ID);
    }

    fn request_callback(&mut self, millis: usize, id: xous::MessageId) {
        self.ticktimer_cb
            .get_or_insert_with(|| TicktimerCallback::new(self.sid).unwrap())
            .request(millis, id, 0);
    }
}

impl server::ServerMessages for WorkerServer {
    const NAME: &str = "";

    fn messages() -> &'static [server::MessageDef<Self>] { &[] }
}

impl server::Server for WorkerServer {}

impl server::ScalarHandler<Wake> for WorkerServer {
    fn handle(&mut self, _msg: Wake, _sender: xous::PID, _context: &mut server::ServerContext<Self>) {
        let mut timer_dirty = false;

        while let Ok(task) = self.rx.try_recv() {
            match task {
                WorkerEvent::Task { task, .. } => {
                    task.run();
                }
                WorkerEvent::Register { mut init } => match init(self) {
                    None => {
                        self.retry_queue.push_back(init);
                        // done every time bc check in method already prevents duplicate callbacks
                        self.request_retry_cb();
                    }
                    Some(()) => {}
                },
                WorkerEvent::Timer { timer } => {
                    self.timers.push(timer);
                    timer_dirty = true;
                }
            }
        }

        if timer_dirty {
            self.process_timers();
        }
    }
}

impl server::ScalarHandler<Shutdown> for WorkerServer {
    fn handle(&mut self, _msg: Shutdown, _sender: xous::PID, context: &mut server::ServerContext<Self>) {
        log::debug!("shutting down worker");
        context.shutdown();
    }
}

impl server::ScalarHandler<RetryCallback> for WorkerServer {
    fn handle(&mut self, _msg: RetryCallback, _sender: xous::PID, _context: &mut ServerContext<Self>) {
        self.retry_cb_active = false;

        let indices = 0..self.retry_queue.len();
        for _ in indices {
            let mut init = self.retry_queue.pop_front().unwrap();
            match init(self) {
                None => {
                    self.retry_queue.push_back(init);
                }
                Some(()) => {}
            }
        }

        if !self.retry_queue.is_empty() {
            self.request_retry_cb();
        }
    }
}

impl server::ScalarHandler<TimerCallback> for WorkerServer {
    fn handle(&mut self, _msg: TimerCallback, _sender: xous::PID, _context: &mut ServerContext<Self>) {
        self.process_timers();
    }
}

#[cfg(feature = "integration_test")]
mod test_msg {
    use super::*;

    worker_msg!(GetRetryTimerActive, 300);
    impl server::BlockingScalar for GetRetryTimerActive {
        type Response = bool;
    }
    impl server::BlockingScalarHandler<GetRetryTimerActive> for WorkerServer {
        fn handle(
            &mut self,
            _msg: GetRetryTimerActive,
            _sender: xous::PID,
            _context: &mut ServerContext<Self>,
        ) -> <GetRetryTimerActive as server::BlockingScalar>::Response {
            self.retry_cb_active
        }
    }
}

#[cfg(feature = "integration_test")]
pub use test_msg::*;
