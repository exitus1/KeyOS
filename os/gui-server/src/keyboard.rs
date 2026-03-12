// SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use gui_server_api::{consts::DEFAULT_KEYBOARD_HEIGHT, InputMessage, Key, KeyboardKind};
use server::AsScalar;
use {
    crate::{DoubleBufferVMA, Gui},
    log::{debug, error, warn},
    xous::{CID, PID},
};

use crate::BlurBufferState;

const KEYBOARD_ANIMATION_STEP_PX: usize = 30;

pub(crate) struct KeyboardWindow {
    pub(crate) input_cid: CID,
    pub(crate) pid: PID,
    pub(crate) bufs: DoubleBufferVMA,
    pub(crate) blur_state: BlurBufferState,

    /// Tracks if the keyboard has been updated with a new layout
    pub(crate) last_drawn_keyboard_kind: KeyboardKind,

    pub(crate) last_requested_keyboard_kind: KeyboardKind,

    /// True if the last notification sent to the window was "show", false if last notification was "hidden"
    pub(crate) notified_shown: bool,
}

impl KeyboardWindow {
    pub(crate) fn request_caps(&mut self, request_caps: bool) {
        let msg = xous::Message::new_scalar(InputMessage::Custom1 as usize, request_caps as usize, 0, 0, 0);
        xous::send_message(self.input_cid, msg)
            .map_err(|e| error!("Failed to set next character as uppercase: {e:?}"))
            .ok();
    }

    pub(crate) fn notify_input_type(&mut self, input_type: KeyboardKind) {
        log::debug!("Setting keyboard input type: {input_type:?}");
        self.last_requested_keyboard_kind = input_type;
        let msg = xous::Message::new_scalar(InputMessage::Custom3 as usize, input_type as usize, 0, 0, 0);
        xous::send_message(self.input_cid, msg)
            .map_err(|e| error!("Failed to set keyboard type: {e:?}"))
            .ok();
    }

    pub(crate) fn notify_keyboard(&self, msg: InputMessage) {
        log::trace!("Notifying keyboard : {msg:?}");
        let msg = xous::Message::new_scalar(msg as usize, 0, 0, 0, 0);
        if let Err(e) = xous::send_message(self.input_cid, msg) {
            error!("Failed to notify keyboard: {e:?}");
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct KeyboardState {
    pub(crate) kind: KeyboardKind,
    pub state: KeyboardCurrentState,
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum KeyboardCurrentState {
    Hidden,
    SlidingIn { height: usize },
    Showing,
    SlidingOut { height: usize },
}

impl KeyboardState {
    pub fn height(&self) -> Option<usize> {
        match self.state {
            KeyboardCurrentState::Hidden => None,
            KeyboardCurrentState::SlidingIn { height } | KeyboardCurrentState::SlidingOut { height } => {
                Some(height)
            }
            KeyboardCurrentState::Showing => Some(DEFAULT_KEYBOARD_HEIGHT),
        }
    }
}

impl Gui {
    pub(crate) fn show_keyboard_for_an_app(&mut self, pid: PID, keyboard_kind: KeyboardKind) {
        debug!("Requested to show keyboard type {keyboard_kind:?}");
        let Some(app) = self.windows.get_mut(&pid) else {
            warn!("Requested to show the keyboard for an app (PID {pid}) that is not registered");
            return;
        };

        app.keyboard_state.kind = keyboard_kind;
        match &app.keyboard_state.state {
            KeyboardCurrentState::Showing | KeyboardCurrentState::SlidingIn { .. } => {}
            KeyboardCurrentState::Hidden => {
                app.keyboard_state.state = KeyboardCurrentState::SlidingIn { height: 0 }
            }
            KeyboardCurrentState::SlidingOut { height } => {
                app.keyboard_state.state = KeyboardCurrentState::SlidingIn { height: *height }
            }
        }

        if self.active_app_pid() == Some(pid) {
            self.update_keyboard_window();
            self.update_layers();
        }
    }

    pub(crate) fn hide_keyboard_for_an_app(&mut self, pid: PID) {
        debug!("Requested to hide the keyboard");
        let active_pid = self.active_app_pid();
        let Some(app) = self.windows.get_mut(&pid) else {
            warn!("Requested to hide the keyboard for an app (PID {pid}) that is not registered");
            return;
        };

        match &app.keyboard_state.state {
            KeyboardCurrentState::Hidden | KeyboardCurrentState::SlidingOut { .. } => {}
            KeyboardCurrentState::Showing => {
                app.keyboard_state.state =
                    KeyboardCurrentState::SlidingOut { height: DEFAULT_KEYBOARD_HEIGHT }
            }
            KeyboardCurrentState::SlidingIn { height } => {
                app.keyboard_state.state = KeyboardCurrentState::SlidingOut { height: *height }
            }
        }
        if active_pid == Some(pid) {
            self.update_keyboard_window();
            self.update_layers();
        } else {
            app.keyboard_state.state = KeyboardCurrentState::Hidden;
        }
    }

    pub(crate) fn update_keyboard_window(&mut self) {
        let Some(pid) = self.active_app_pid() else { return };
        let Some(window) = self.windows.get_mut(&pid) else { return };
        let Some(keyboard_window) = &mut self.keyboard_window else { return };

        let visible = !matches!(window.keyboard_state.state, KeyboardCurrentState::Hidden);
        if visible && !keyboard_window.notified_shown {
            keyboard_window.notify_keyboard(InputMessage::Visible);
            keyboard_window.notified_shown = true;
        }
        if !visible && keyboard_window.notified_shown {
            keyboard_window.notify_keyboard(InputMessage::Hidden);
            keyboard_window.notified_shown = false;
        }

        if window.keyboard_state.kind != keyboard_window.last_drawn_keyboard_kind {
            match window.keyboard_state.state {
                KeyboardCurrentState::Hidden => {}
                KeyboardCurrentState::Showing | KeyboardCurrentState::SlidingIn { .. } => {
                    keyboard_window.notify_input_type(window.keyboard_state.kind);
                    // Practically hide the window until it is ready to be shown.
                    window.keyboard_state.state = KeyboardCurrentState::SlidingIn { height: 0 };
                }
                KeyboardCurrentState::SlidingOut { .. } => {
                    window.keyboard_state.state = KeyboardCurrentState::Hidden
                }
            }
        }
    }

    pub(crate) fn keyboard_animation_tick(&mut self) {
        let Some(pid) = self.active_app_pid() else { return };
        let Some(window) = self.windows.get_mut(&pid) else { return };
        let Some(keyboard_window) = &mut self.keyboard_window else { return };

        // We have the wrong kind in the display buffer, and we are probably
        // in state SlidingIn {0}, waiting for the right buffer.
        if window.keyboard_state.kind != keyboard_window.last_drawn_keyboard_kind {
            return;
        }
        match &mut window.keyboard_state.state {
            KeyboardCurrentState::SlidingOut { height } => {
                if *height > KEYBOARD_ANIMATION_STEP_PX {
                    *height -= KEYBOARD_ANIMATION_STEP_PX;
                } else {
                    window.keyboard_state.state = KeyboardCurrentState::Hidden;
                }
            }
            KeyboardCurrentState::SlidingIn { height } => {
                if *height < DEFAULT_KEYBOARD_HEIGHT - KEYBOARD_ANIMATION_STEP_PX {
                    *height += KEYBOARD_ANIMATION_STEP_PX;
                } else {
                    window.keyboard_state.state = KeyboardCurrentState::Showing;
                }
            }
            _ => {}
        }
        self.update_keyboard_window();
    }

    pub(crate) fn swap_keyboard_bufs(&mut self, pid: PID) -> bool {
        if self.keyboard_window.as_ref().map(|w| w.pid != pid).unwrap_or(false) {
            return false;
        }
        let Some(keyboard_window) = &mut self.keyboard_window else {
            warn!("no keyboard window");
            return false;
        };
        keyboard_window.bufs = *keyboard_window.bufs.swap();

        // We finally got the right keyboard type, let's start sliding it in
        if keyboard_window.last_drawn_keyboard_kind != keyboard_window.last_requested_keyboard_kind {
            keyboard_window.last_drawn_keyboard_kind = keyboard_window.last_requested_keyboard_kind;
            self.keyboard_animation_tick();
        }
        true
    }

    /// Sends the key press/release event to the currently active app.
    pub(crate) fn dispatch_key_event(&mut self, is_pressed: bool, key: Key) {
        let [arg1, arg2] = key.as_scalar();

        let input_msg_kind = if is_pressed { InputMessage::KeyPress } else { InputMessage::KeyRelease };

        debug!("Sending key {}: {:?}", if is_pressed { "press" } else { "release" }, key);

        self.with_active_app_mut(|app| {
            let msg = xous::Message::new_scalar(input_msg_kind as usize, arg1 as usize, arg2 as usize, 0, 0);
            if let Err(e) = xous::send_message(app.input_cid, msg) {
                error!("Failed to send the input event to the app: {e:?}");
            }
        });
    }
}

impl Default for KeyboardState {
    fn default() -> Self {
        KeyboardState { kind: KeyboardKind::Alphanumeric, state: KeyboardCurrentState::Hidden }
    }
}
