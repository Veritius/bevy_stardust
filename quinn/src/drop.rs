use std::{future::Future, task::Poll};
use crossbeam_channel::{Receiver, Sender};
use futures_lite::FutureExt;

pub fn single<V>() -> (SendSingle<V>, RecvSingle<V>) {
    let (tx, rx) = crossbeam_channel::bounded(1);
    return (SendSingle(tx), RecvSingle(rx));
}

pub fn drop() -> (DropAlerter, DropListener) {
    let (tx, rx) = single::<()>();
    return (DropAlerter(tx), DropListener(rx));
}

/// Can send a single message once to a paired [`RecvSingle`].
pub(crate) struct SendSingle<V>(Sender<V>);

impl<V> SendSingle<V> {
    pub fn send(self, message: V) {
        let _ = self.0.send(message);
    }
}

/// Receives a single message once from a paired [`SendSingle`].
#[derive(Clone)]
pub(crate) struct RecvSingle<V>(Receiver<V>);

impl<V> Future for RecvSingle<V> {
    type Output = Option<V>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        Poll::Ready(self.0.recv().ok())
    }
}

/// Alerts all paired [`DropListener`] objects when dropped.
pub(crate) struct DropAlerter(SendSingle<()>);

impl Drop for DropAlerter {
    fn drop(&mut self) {
        let _ = self.0.0.send(());
    }
}

/// Receives a single message once when its paired [`DropAlerter`] is dropped.
#[derive(Clone)]
pub(crate) struct DropListener(RecvSingle<()>);

impl Future for DropListener {
    type Output = ();

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Self::Output> {
        match self.0.poll(cx) {
            Poll::Ready(_) => Poll::Ready(()),
            Poll::Pending => Poll::Pending,
        }
    }
}