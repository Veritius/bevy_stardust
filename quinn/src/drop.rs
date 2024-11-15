use std::{future::Future, task::Poll};
use crossbeam_channel::{Receiver, RecvError, Sender};
use futures_lite::FutureExt;

pub fn single() -> (SendSingle, RecvSingle) {
    let (tx, rx) = crossbeam_channel::bounded(1);
    return (SendSingle(tx), RecvSingle(rx));
}

pub fn drop() -> (DropAlerter, DropListener) {
    let (tx, rx) = single();
    return (DropAlerter(tx), DropListener(rx));
}

/// Can send a single message once to a paired [`RecvSingle`].
pub(crate) struct SendSingle(Sender<()>);

impl SendSingle {
    pub fn send(self) {
        let _ = self.0.send(());
    }
}

/// Receives a single message once from a paired [`SendSingle`].
#[derive(Clone)]
pub(crate) struct RecvSingle(Receiver<()>);

impl Future for RecvSingle {
    type Output = Result<(), RecvError>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        Poll::Ready(self.0.recv())
    }
}

/// Alerts all paired [`DropListeners`] when dropped.
pub(crate) struct DropAlerter(SendSingle);

impl Drop for DropAlerter {
    fn drop(&mut self) {
        let _ = self.0.0.send(());
    }
}

/// Receives a single message once when its paired [`DropAlerter`] is dropped.
#[derive(Clone)]
pub(crate) struct DropListener(RecvSingle);

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