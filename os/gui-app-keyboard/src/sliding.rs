// SPDX-FileCopyrightText: 2026 Foundation Devices, Inc. <hello@foundationdevices.com>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Cursor-sliding mode: drag from the spacebar to move the cursor by emitting arrow keys.

use gui_server_api::Key;

/// Minimum horizontal drag distance (in pixels) from the touch origin to activate sliding.
const ACTIVATION_THRESHOLD: f32 = 16.0;

/// Horizontal pixels of movement per emitted arrow key.
const PIXELS_PER_KEY: f32 = 20.0;

/// Events produced by [`CursorSliding::on_moved`].
pub enum SlideEvent {
    /// Sliding state exists but nothing to do yet.
    Idle,
    /// The drag just crossed the activation threshold.
    Activated,
    /// An arrow key should be emitted.
    EmitKey(Key),
}

#[derive(Debug, Clone, Copy)]
struct State {
    /// Sliding origin, shifted by `PIXELS_PER_KEY` on each emitted key.
    origin_x: f32,
    origin_y: f32,
    active: bool,
}

/// Self-contained cursor-sliding state machine.
pub struct CursorSliding {
    state: Option<State>,
}

impl CursorSliding {
    pub fn new() -> Self { Self { state: None } }

    /// Whether the sliding gesture is currently active (threshold exceeded).
    pub fn is_active(&self) -> bool { self.state.map_or(false, |s| s.active) }

    /// Begin tracking a potential slide from (`x`, `y`).
    pub fn start(&mut self, x: f32, y: f32) {
        self.state = Some(State { origin_x: x, origin_y: y, active: false });
    }

    /// Reset all sliding state. Returns `true` if sliding was active.
    pub fn reset(&mut self) -> bool {
        let was_active = self.is_active();
        self.state = None;
        was_active
    }

    /// Process a drag event.
    ///
    /// Returns `None` when no sliding state exists (the caller should fall
    /// through to normal drag handling). Returns `Some(event)` when the
    /// sliding state machine consumed the event.
    pub fn on_moved(&mut self, x: f32, y: f32) -> Option<SlideEvent> {
        let state = self.state.as_mut()?;

        let dx = x - state.origin_x;
        let dy = (y - state.origin_y).abs();

        if !state.active {
            // Only activate when the horizontal displacement dominates
            // so that vertical drags don't cancel the space key.
            return Some(if dx.abs() > ACTIVATION_THRESHOLD && dx.abs() > dy {
                state.active = true;
                SlideEvent::Activated
            } else {
                SlideEvent::Idle
            });
        }

        if dx.abs() >= PIXELS_PER_KEY {
            let key = if dx > 0.0 { Key::CursorRight } else { Key::CursorLeft };
            // Shift origin toward current position, keeping the leftover.
            state.origin_x += dx.signum() * PIXELS_PER_KEY;
            Some(SlideEvent::EmitKey(key))
        } else {
            Some(SlideEvent::Idle)
        }
    }
}
