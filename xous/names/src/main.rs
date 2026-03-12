use std::collections::{HashMap, HashSet};

use app_manifest::Manifest;
use log::{error, info};
use num_traits::FromPrimitive;
use xous::{AppId, MessageEnvelope, MessageId};
use xous_api_names::*;

include!("system_manifests.rs");

#[derive(PartialEq)]
#[repr(C)]
enum ConnectSuccess {
    /// The server connection was successfully made
    Connected(xous::CID /* Connection ID */),

    /// There is no server with that name -- block this message
    Wait,
}

#[derive(Default)]
struct NameServer {
    waiting_connections: Vec<MessageEnvelope>,
    name_table: HashMap<String, xous::SID>,
    message_name_to_id: HashMap<(String, String), Vec<MessageId>>,
    register_permissions: HashMap<AppId, HashSet<String>>,
    connect_permissions: HashMap<AppId, HashMap<String, Vec<MessageId>>>,
}

impl NameServer {
    fn process_manifest_servers(&mut self, manifest: &Manifest) -> Result<(), xous::Error> {
        let app_id = AppId(manifest.app_id_bytes());
        for (server_name, messages) in manifest.servers.iter() {
            self.register_permissions.entry(app_id).or_default().insert(server_name.clone());
            for (message_name, message) in messages {
                self.message_name_to_id.insert((server_name.clone(), message_name.clone()), vec![message.id]);
            }
        }
        Ok(())
    }

    fn process_manifest_permissions(&mut self, manifest: &Manifest) -> Result<(), xous::Error> {
        let app_id = AppId(manifest.app_id_bytes());
        let app_permissions = self.connect_permissions.entry(app_id).or_default();
        for (server_name, messages) in manifest.permissions.iter() {
            let server_permissions = app_permissions.entry(server_name.clone()).or_default();
            for message_name in messages {
                let Some(message_id) =
                    self.message_name_to_id.get(&(server_name.clone(), message_name.clone()))
                else {
                    log::error!("Rejected manifest for `{}`, couldn't find `{message_name}` message for `{server_name}` server", manifest.app_name_en());
                    return Err(xous::Error::ServerNotFound);
                };

                server_permissions.extend_from_slice(message_id);
            }
            server_permissions.sort_unstable();
        }
        Ok(())
    }

    /// Connect to the server named in the message. If the server exists, attempt the connection
    /// and return either the connection ID or an error.
    ///
    /// If the server does not exist, return `Ok(None)`
    fn connect_impl(&self, msg: &MessageEnvelope) -> Result<ConnectSuccess, xous::Error> {
        let app_id = xous::get_app_id(msg.sender.pid().ok_or(xous::Error::ProcessNotFound)?)?
            .ok_or(xous::Error::ProcessNotFound)?;
        let server_name = name_from_msg(msg, 0)?;
        let sender_pid = msg.sender.pid().expect("kernel provided us a PID of None");
        log::trace!("BlockingConnect request for '{}' for process {:?}", server_name, sender_pid);

        let default_permissions = HashMap::default();
        let app_permissions = self.connect_permissions.get(&app_id).unwrap_or(&default_permissions);
        // If the server already exists, attempt to make the connection. The connection can
        // only succeed if the server is in the name_table.
        if let Some(server_sid) = self.name_table.get(server_name) {
            log::trace!(
                "Found entry in the table (sid: {:?}) -- attempting to call connect_for_process()",
                server_sid,
            );
            let no_perms = Vec::new();
            let Some(connection_permissions) = app_permissions
                .get(server_name)
                .or_else(|| self.register_permissions.get(&app_id)?.get(server_name).map(|_| &no_perms))
            else {
                log::warn!("App {app_id:02x?} tried to connect to {server_name} without permissions.");
                return Err(xous::Error::AccessDenied);
            };
            let result = xous::connect_for_process(sender_pid, *server_sid);
            match result {
                Ok(connection_id) => {
                    for permission in connection_permissions {
                        xous::allow_messages_on_connection(
                            sender_pid,
                            connection_id,
                            *permission..*permission + 1,
                        )
                        .expect("permissions");
                    }

                    log::trace!(
                        "Connected to '{}' for process {:?} with CID {}",
                        server_name,
                        sender_pid,
                        connection_id,
                    );
                    return Ok(ConnectSuccess::Connected(connection_id));
                }
                Err(e) => {
                    log::error!("error when making connection, perhaps the server crashed? {e:?}");
                    return Err(e);
                }
            }
        }

        // There is no connection, so block the sender
        log::trace!("No server currently registered to '{}', blocking...", server_name);
        Ok(ConnectSuccess::Wait)
    }

    fn register_impl(&mut self, msg: &MessageEnvelope) -> Result<String, xous::Error> {
        let sid = sid_from_msg(msg)?;
        let server_name = name_from_msg(msg, 16)?.to_string();
        let app_id = xous::get_app_id(msg.sender.pid().ok_or(xous::Error::ProcessNotFound)?)?
            .ok_or(xous::Error::ProcessNotFound)?;
        log::trace!("registration request for '{}'", server_name);
        if !self.register_permissions.get(&app_id).ok_or(xous::Error::AccessDenied)?.contains(&server_name) {
            return Err(xous::Error::AccessDenied);
        }
        if self.name_table.contains_key(&server_name) {
            return Err(xous::Error::MemoryInUse);
        }
        self.name_table.insert(server_name.clone(), sid);
        log::trace!("request successful, SID is {:?}", sid);
        Ok(server_name)
    }

    fn wake_waiting_connections(&mut self, server_name: &str) {
        // See if we have any requests matching this server ID. If so, make the
        // connection and don't retain it.
        let mut i = 0;
        while i < self.waiting_connections.len() {
            if name_from_msg(&self.waiting_connections[i], 0) == Ok(server_name) {
                let msg = self.waiting_connections.swap_remove(i);
                match self.connect_impl(&msg) {
                    Err(e) => respond_error(msg, e),
                    Ok(ConnectSuccess::Connected(cid)) => respond_connect_success(msg, cid),
                    Ok(ConnectSuccess::Wait) => {
                        panic!("message connection attempt resulted in `Wait` even though it ought to exist");
                    }
                }
            } else {
                i += 1;
            }
        }
    }

    fn process_system_manifests(&mut self) {
        let parsed_manifests: Vec<Manifest> = SYSTEM_MANIFESTS
            .iter()
            .map(|m| Manifest::try_from_bytes(m.as_bytes()).expect("Could not parse system manifest"))
            .collect();
        // Two stage processing, so that permissions can refer to message IDs declared in later manifests.
        for manifest in &parsed_manifests {
            if self.process_manifest_servers(&manifest).is_err() {
                // Xtask should have prevented this from even building, so it's panic-worthy.
                panic!("Could not parse servers or groups in manifest for appID={:?}.", manifest.app_id);
            };
        }
        for manifest in parsed_manifests {
            if self.process_manifest_permissions(&manifest).is_err() {
                // Xtask should have prevented this from even building, so it's panic-worthy.
                panic!("Could not parse permissions in manifest for appID={:?}.", manifest.app_id);
            }
        }
    }

    fn add_manifest_impl(&mut self, msg: &MessageEnvelope) -> Result<(), xous::Error> {
        let app_id = xous::get_app_id(msg.sender.pid().ok_or(xous::Error::ProcessNotFound)?)?
            .ok_or(xous::Error::ProcessNotFound)?;
        if !self
            .connect_permissions
            .get(&app_id)
            .ok_or(xous::Error::AccessDenied)?
            .get("os/nameserver")
            .ok_or(xous::Error::AccessDenied)?
            .contains(&(api::Opcode::AddManifest as usize))
        {
            return Err(xous::Error::AccessDenied);
        }
        let msg = msg.body.memory_message().ok_or(xous::Error::InvalidArguments)?;
        let valid_bytes = msg.valid.map(|v| v.get()).unwrap_or_else(|| msg.buf.len());
        log::trace!("Parsing manifest, len={valid_bytes}");

        let manifest = Manifest::try_from_bytes(&msg.buf.as_slice()[..valid_bytes]).map_err(|e| {
            log::error!("Error parsing manifest: {e:?}");
            xous::Error::ParseError
        })?;

        log::trace!("Parsed manifest for `{}`", manifest.app_name_en());

        self.process_manifest_servers(&manifest)?;
        self.process_manifest_permissions(&manifest)?;
        Ok(())
    }

    fn run(&mut self) -> ! {
        xous::set_thread_priority(xous::ThreadPriority::Highest).unwrap();
        // TODO: Only allow privileged clients to add new permissions (SFT-5024, SFT-5025)
        let name_server =
            xous::create_server_with_sid(xous::SID::from_bytes(b"xous-name-server").unwrap(), 0..8)
                .expect("Couldn't create xous-name-server");

        info!("Starting");
        self.process_system_manifests();

        loop {
            // Init logging (may fail before logging server has started)
            log_server::init(env!("CARGO_CRATE_NAME")).ok();
            log::set_max_level(log::LevelFilter::Info);

            let msg = xous::receive_message(name_server).unwrap();
            log::trace!("received message: {:?}", msg);
            if !msg.body.is_blocking() {
                continue;
            }
            if !msg.body.has_memory() {
                xous::return_scalar(msg.sender, 0).unwrap();
                continue;
            }
            match FromPrimitive::from_usize(msg.body.id()) {
                Some(api::Opcode::Register) => match self.register_impl(&msg) {
                    Ok(name) => {
                        respond_simple_success(msg);
                        self.wake_waiting_connections(&name);
                    }
                    Err(err) => respond_error(msg, err),
                },
                Some(api::Opcode::BlockingConnect) | Some(api::Opcode::TryConnect) => {
                    match self.connect_impl(&msg) {
                        Err(e) => respond_error(msg, e),
                        Ok(ConnectSuccess::Connected(cid)) => respond_connect_success(msg, cid),
                        Ok(ConnectSuccess::Wait) => {
                            if msg.body.id() == api::Opcode::TryConnect as usize {
                                respond_error(msg, xous::Error::ServerNotFound);
                            } else {
                                // Push waiting connections here, which will prevent it from getting
                                // dropped and responded to.
                                self.waiting_connections.push(msg);
                            }
                        }
                    }
                }
                Some(api::Opcode::AddManifest) => match self.add_manifest_impl(&msg) {
                    Ok(()) => respond_simple_success(msg),
                    Err(err) => respond_error(msg, err),
                },
                None => {
                    error!("couldn't decode message: {:?}", msg);
                }
            }
        }
    }
}

fn sid_from_msg(env: &MessageEnvelope) -> Result<xous::SID, xous::Error> {
    let msg = env.body.memory_message().ok_or(xous::Error::InvalidArguments)?;
    Ok(xous::SID::from_u32(
        msg.buf.as_slice()[0],
        msg.buf.as_slice()[1],
        msg.buf.as_slice()[2],
        msg.buf.as_slice()[3],
    ))
}

fn name_from_msg(env: &MessageEnvelope, offset: usize) -> Result<&str, xous::Error> {
    let msg = env.body.memory_message().ok_or(xous::Error::InvalidArguments)?;
    let valid_bytes = msg.valid.map(|v| v.get()).unwrap_or_else(|| msg.buf.len());
    if valid_bytes > msg.buf.len() {
        log::error!("valid bytes exceeded entire buffer length");
        return Err(xous::Error::InvalidString);
    }
    // Safe because we've already validated that it's a valid range
    let str_slice =
        unsafe { core::slice::from_raw_parts(msg.buf.as_ptr().wrapping_add(offset), valid_bytes) };
    let name_string = core::str::from_utf8(str_slice).map_err(|_| xous::Error::InvalidString)?;

    Ok(name_string)
}

fn respond_error(mut msg: MessageEnvelope, error: xous::Error) {
    let mem = msg.body.memory_message_mut().unwrap();
    mem.buf.as_slice_mut::<u32>()[0] = 1;
    mem.buf.as_slice_mut::<u32>()[1] = error as u32;
    mem.valid = None;
    mem.offset = None;
}

fn respond_connect_success(mut msg: MessageEnvelope, cid: xous::CID) {
    let mem = msg.body.memory_message_mut().unwrap();
    mem.buf.as_slice_mut::<u32>()[0] = 0;
    mem.buf.as_slice_mut::<u32>()[1] = cid as u32;
    mem.valid = None;
    mem.offset = None;
}

fn respond_simple_success(mut msg: MessageEnvelope) {
    let mem = msg.body.memory_message_mut().unwrap();
    mem.buf.as_slice_mut::<u32>()[0] = 0;
    mem.valid = None;
    mem.offset = None;
}

fn main() -> ! { NameServer::default().run() }
