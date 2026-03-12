// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
    pin::Pin,
    sync::Weak,
    task::{Context, Poll},
    time::{Duration, Instant},
};

use crate::implementation::{Timer, WorkerApiInner, WorkerEvent};

#[must_use]
pub struct Sleep {
    expires_at: Instant,
    handle: Weak<WorkerApiInner>,
    has_woken: bool,
}

impl Sleep {
    pub(crate) fn new(duration: Duration, handle: Weak<WorkerApiInner>) -> Self {
        Self { expires_at: Instant::now() + duration, handle, has_woken: false }
    }
}

impl std::future::Future for Sleep {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.has_woken && self.expires_at <= Instant::now() {
            Poll::Ready(())
        } else {
            if let Some(handle) = self.handle.upgrade() {
                handle.queue_event(WorkerEvent::Timer {
                    timer: Timer { instant: self.expires_at, waker: cx.waker().clone() },
                });
            }
            self.has_woken = true;
            Poll::Pending
        }
    }
}
