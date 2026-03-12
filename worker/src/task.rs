// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
    pin::{pin, Pin},
    task::{Context, Poll},
};

#[must_use = "dropping this handle will cancel the task. use `TaskHandle::detach()` to have the task run in the background."]
pub struct TaskHandle<T> {
    pub(crate) task: async_task::Task<T>,
}

impl<T> From<async_task::Task<T>> for TaskHandle<T> {
    fn from(task: async_task::Task<T>) -> Self { Self { task } }
}

impl<T> TaskHandle<T> {
    /// Cancels the task and waits for it to stop running.
    /// Returns the task's output if it was completed just before it got canceled, or [`None`] if
    /// it didn't complete.
    pub async fn cancel(self) -> Option<T> { self.task.cancel().await }

    /// Detaches the task to let it keep running in the background.
    pub fn detach(self) { self.task.detach(); }

    /// Returns `true` if the current task is finished.
    pub fn is_finished(&self) -> bool { self.task.is_finished() }
}

impl<T> Future for TaskHandle<T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        pin!(&mut self.task).poll(cx)
    }
}
