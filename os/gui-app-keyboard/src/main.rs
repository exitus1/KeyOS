// SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

mod cache;
#[macro_use]
pub mod colors;
mod drawing;
mod font;
mod key_slot;
mod keyboard;
mod keys;
mod layout;
mod overlay;
mod sliding;

use std::{rc::Rc, sync::atomic::AtomicBool};

use gui_server_api::{
    consts::{DEFAULT_KEYBOARD_HEIGHT, SCREEN_WIDTH},
    touch::{Touch, TouchKind},
    InputMessage, KeyboardKind, Vsync,
};
use num_traits::FromPrimitive;
use tiny_skia::PixmapMut;
use worker::WorkerHandle;
use xous::MessageEnvelope;
use xous_api_ticktimer::TicktimerCallback;

use crate::{cache::refresh_cache, keyboard::KeyboardState};

gui_server_api::use_api!();
haptics::use_api!();
#[cfg(not(feature = "recovery-os"))]
settings::use_api!();

static IS_DARK: AtomicBool = AtomicBool::new(true);

const WIDTH: usize = SCREEN_WIDTH;
const HEIGHT: usize = DEFAULT_KEYBOARD_HEIGHT;
fn main() -> ! {
    log_server::init_wait(env!("CARGO_CRATE_NAME")).unwrap();
    log::set_max_level(log::LevelFilter::Info);

    let (gui, framebuffer) =
        GuiApi::register(gui_server_api::AppKind::Keyboard, "keyboard", WIDTH * HEIGHT * 4)
            .expect("can't register app UI");

    let mut bufs = framebuffer.into_bufs().expect("init app framebuffer");
    let haptics_api = Rc::new(HapticsApi::default());
    let long_press_callback = TicktimerCallback::new(gui.sid()).unwrap();
    let mut state =
        keyboard::KeyboardState::new(KeyboardKind::Numbers, haptics_api.clone(), long_press_callback);

    let background_worker = WorkerHandle::default();
    background_worker
        .spawn(async { xous::set_thread_priority(xous::ThreadPriority::AppBackground1).unwrap() })
        .detach();
    #[cfg(not(feature = "recovery-os"))]
    subscribe_theme_change(&background_worker);

    #[cfg(feature = "recovery-os")]
    background_worker.spawn(async { refresh_cache() }).detach();

    loop {
        // Process all other pending messages first
        while let Some((event, msg)) = gui.try_receive_input() {
            process_input(event, msg, &mut state, &gui);
        }

        let work_fb = bufs.work_buf as *mut u8;
        let work_fb = unsafe { std::slice::from_raw_parts_mut(work_fb, WIDTH * HEIGHT * 4) };
        state.draw(&mut PixmapMut::from_bytes(work_fb, WIDTH as u32, HEIGHT as u32).unwrap());

        #[cfg(keyos)]
        xous::syscall::flush_cache(
            unsafe { xous::MemoryRange::new(bufs.work_buf, WIDTH * HEIGHT * 4).unwrap() },
            xous::CacheOperation::Clean,
        )
        .expect("clean cache");

        if let Some(_swap_time) = gui.swap_buffers(Vsync::Wait).expect("swap buffers") {
            bufs.swap();
        } else {
            log::warn!("swap_buffers() was unsuccessful");
        }

        // Block until next message
        if let Ok((event, msg)) = gui.receive_input() {
            process_input(event, msg, &mut state, &gui);
        }
    }
}

#[cfg(not(feature = "recovery-os"))]
fn subscribe_theme_change(background_worker: &WorkerHandle) {
    let mut theme_updates = background_worker
        .subscribe_scalar::<settings_permissions::SettingsPermissions, _>(
            settings::messages::SubscribeSystemTheme,
        );
    background_worker
        .spawn(async move {
            while let Some(theme) = theme_updates.next().await {
                IS_DARK
                    .store(theme == settings::global::SystemTheme::Dark, std::sync::atomic::Ordering::SeqCst);
                refresh_cache();
            }
        })
        .detach();
}

fn process_input(event: InputMessage, msg: MessageEnvelope, state: &mut KeyboardState, gui: &GuiApi) {
    match event {
        InputMessage::Touch => {
            if let Some(touch) = Touch::try_from_input_message(&msg.body) {
                match touch.kind {
                    TouchKind::Press => {
                        state.on_pressed(touch.x as f32, touch.y as f32);
                    }
                    TouchKind::Drag => {
                        if let Some(key) = state.on_moved(touch.x as f32, touch.y as f32) {
                            gui.key_pressed(key).expect("gui-server api unavailable");
                            gui.key_released(key).expect("gui-server api unavailable");
                        }
                    }
                    TouchKind::Release => {
                        if let Some(key) = state.on_released(touch.x as f32, touch.y as f32) {
                            gui.key_pressed(key).expect("gui-server api unavailable");
                            gui.key_released(key).expect("gui-server api unavailable");
                        }
                    }
                };
            }
        }
        InputMessage::Hidden => {
            state.shift_state_request(false);
        }
        InputMessage::Custom1 => {
            if let Some(xous::ScalarMessage { arg1, .. }) = msg.body.scalar_message() {
                log::debug!("Got request to set caps for next character: {arg1:?}");
                state.shift_state_request(*arg1 != 0);
            }
        }
        InputMessage::Custom3 => {
            if let Some(xous::ScalarMessage { arg1, .. }) = msg.body.scalar_message() {
                if let Some(input_type) = KeyboardKind::from_usize(*arg1) {
                    log::info!("changing input type to {input_type:?}");
                    state.set_kind(input_type)
                }
            }
        }
        InputMessage::Custom4 => {
            state.on_long_press();
        }
        _ => (),
    }
}
