// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
    panic::Location,
    pin::{pin, Pin},
    task::{Context, Poll},
};

use server::xous;

pub struct Response<M> {
    pub(crate) rx: oneshot::Receiver<Result<xous::MessageEnvelope, xous::Error>>,
    pub(crate) decode: fn(Result<xous::MessageEnvelope, xous::Error>, &'static Location<'static>) -> M,
    pub(crate) location: &'static Location<'static>,
}

impl<M> Response<M> {
    pub fn is_finished(&self) -> bool { self.rx.has_message() }

    pub fn block_on(self) -> M {
        let envelope = self.rx.recv().unwrap();
        (self.decode)(envelope, self.location)
    }
}

impl<M> Future for Response<M> {
    type Output = M;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match pin!(&mut self.rx).poll(cx) {
            Poll::Ready(envelope) => {
                let result = (self.decode)(envelope.unwrap(), self.location);
                Poll::Ready(result)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}
