// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
    pin::Pin,
    task::{Context, Poll},
};

use server::xous;

pub struct Subscription<M> {
    pub(crate) rx: async_channel::Receiver<xous::MessageEnvelope>,
    pub(crate) decode: fn(xous::MessageEnvelope) -> M,
}

impl<M> Subscription<M> {
    pub async fn next(&mut self) -> Option<M> {
        self.rx.recv().await.ok().map(|envelope| (self.decode)(envelope))
    }
}

impl<M> futures_lite::Stream for Subscription<M> {
    type Item = M;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let rx = unsafe { self.as_mut().map_unchecked_mut(|s| &mut s.rx) };
        match rx.poll_next(cx) {
            Poll::Ready(Some(envelope)) => Poll::Ready(Some((self.decode)(envelope))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<T> Clone for Subscription<T> {
    fn clone(&self) -> Self { Self { rx: self.rx.clone(), decode: self.decode.clone() } }
}
