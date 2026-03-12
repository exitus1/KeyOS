// SPDX-FileCopyrightText: 2026 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

mod parse;
mod serial;
mod state;
mod tui;

use std::sync::mpsc;
use std::sync::mpsc::RecvTimeoutError;
use std::thread;
use std::time::Duration;

use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::parse::{parse_payload, ParseItem};
use crate::serial::{start_serial_thread, SerialCommand};
use crate::state::State;

const MAX_EVENTS_PER_FRAME: usize = 32;

#[derive(Debug)]
pub enum AppEvent {
    Input(Event),
    Serial(SerialEvent),
}

#[derive(Debug)]
pub enum SerialEvent {
    Payload(String),
    Status(String),
}

#[derive(Parser)]
#[command(name = "keyos-log-viewer")]
#[command(about = "View keyOS logs from serial port")]
#[command(version)]
struct Args {
    /// Serial port (e.g., /dev/ttyUSB0, COM3)
    #[arg(short, long, value_name = "PORT")]
    port: String,

    /// Baud rate (default: 115200)
    #[arg(short, long, default_value = "115200")]
    baud: u32,

    /// Reconnect timeout in seconds (default: 3)
    #[arg(short, long, default_value = "3")]
    timeout: u64,
}

fn main() {
    let args = Args::parse();
    let port = args.port.clone();

    let (event_tx, event_rx) = mpsc::channel::<AppEvent>();
    let (serial_tx, serial_rx) = mpsc::channel::<SerialCommand>();

    thread::spawn({
        let event_tx = event_tx.clone();
        move || {
            while let Ok(event) = event::read() {
                if event_tx.send(AppEvent::Input(event)).is_err() {
                    break;
                }
            }
        }
    });

    start_serial_thread(
        args.port.clone(),
        args.baud,
        Duration::from_secs(args.timeout),
        event_tx.clone(),
        serial_rx,
    );

    let mut state = State::new(serial_tx.clone(), port);
    state.log.refresh_filters();

    enable_raw_mode().unwrap();
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|f| tui::draw(f, &mut state)).unwrap();
    run_loop(&event_rx, &mut state, &mut terminal);

    disable_raw_mode().unwrap();
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture).unwrap();
    terminal.show_cursor().unwrap();
}

fn run_loop(
    event_rx: &mpsc::Receiver<AppEvent>,
    state: &mut State,
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
) {
    loop {
        match event_rx.recv_timeout(Duration::from_millis(250)) {
            Ok(event) => {
                if apply_event(state, event) {
                    return;
                }
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => return,
        }

        for _ in 0..MAX_EVENTS_PER_FRAME {
            match event_rx.try_recv() {
                Ok(event) => {
                    if apply_event(state, event) {
                        return;
                    }
                }
                Err(_) => break,
            }
        }

        state.handle_tick();
        terminal.draw(|f| tui::draw(f, state)).unwrap();
    }
}

fn apply_event(state: &mut State, event: AppEvent) -> bool {
    match event {
        AppEvent::Input(event) => state.handle_input(event),
        AppEvent::Serial(event) => {
            match event {
                SerialEvent::Payload(payload) => {
                    if let Some(parsed) = parse_payload(&payload, &state.log.entries) {
                        match parsed {
                            ParseItem::Log(record) => state.log.push_entry(record),
                            ParseItem::ProcessSnapshot(snapshot) => {
                                state.process.handle_process_snapshot(snapshot);
                            }
                        }
                    }
                }
                SerialEvent::Status(status) => {
                    state.status_text = status;
                }
            }
            false
        }
    }
}
