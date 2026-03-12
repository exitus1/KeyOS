// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::time::Duration;

pub struct ExponentialBackoff {
    current: Duration,
    max: Duration,
    attempts: u32,
    max_attempts: u32,
}

impl ExponentialBackoff {
    pub const fn new(initial: Duration, max: Duration, max_attempts: u32) -> Self {
        Self { current: initial, max, attempts: 0, max_attempts }
    }

    pub fn has_next(&self) -> bool { self.attempts < self.max_attempts }
}

impl Iterator for ExponentialBackoff {
    type Item = Duration;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.has_next() {
            return None;
        }

        let duration = self.current;
        self.attempts += 1;

        // Double the delay for next time, but don't exceed max
        self.current =
            Duration::from_millis((self.current.as_millis() as u64 * 2).min(self.max.as_millis() as u64));

        Some(duration)
    }
}
