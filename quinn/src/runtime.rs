use std::{fmt::Debug, future::Future, io::ErrorKind, sync::Arc, task::Poll, time::Instant};
use bevy::tasks::{futures_lite::ready, ComputeTaskPool};
use quinn::{udp::UdpSocketState, AsyncTimer, AsyncUdpSocket, Runtime, UdpPoller};
use tokio::{io::Interest, net::UdpSocket, time::Sleep};

#[derive(Debug)]
pub(crate) struct BevyRuntime;

impl Runtime for BevyRuntime {
    fn new_timer(&self, i: std::time::Instant) -> std::pin::Pin<Box<dyn quinn::AsyncTimer>> {
        Box::pin(Timer(tokio::time::sleep(i.duration_since(Instant::now()))))
    }

    fn spawn(&self, future: std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>) {
        ComputeTaskPool::get().spawn(future);
    }

    fn wrap_udp_socket(&self, t: std::net::UdpSocket) -> std::io::Result<std::sync::Arc<dyn quinn::AsyncUdpSocket>> {
        return Ok(Arc::new(Socket {
            state: UdpSocketState::new((&t).into())?,
            socket: UdpSocket::from_std(t)?,
        }));
    }
}

#[derive(Debug)]
pub(crate) struct Timer(Sleep);

// SAFETY: Timer is structurally pinned
// https://doc.rust-lang.org/std/pin/index.html#choosing-pinning-to-be-structural-for-field
impl AsyncTimer for Timer {
    fn reset(self: std::pin::Pin<&mut Self>, i: std::time::Instant) {
        let sleep = unsafe { self.map_unchecked_mut(|v| &mut v.0) };
        Sleep::reset(sleep, i.into());
    }

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context) -> std::task::Poll<()> {
        let sleep = unsafe { self.map_unchecked_mut(|v| &mut v.0) };
        Future::poll(sleep, cx)
    }
}

#[derive(Debug)]
pub(crate) struct Socket {
    state: UdpSocketState,
    socket: UdpSocket,
}

impl AsyncUdpSocket for Socket {
    fn create_io_poller(self: std::sync::Arc<Self>) -> std::pin::Pin<Box<dyn quinn::UdpPoller>> {
        Box::pin(SocketPoller::new(move || {
            let socket = self.clone();
            async move { socket.socket.writable().await }
        }))
    }

    fn try_send(&self, transmit: &quinn::udp::Transmit) -> std::io::Result<()> {
        match self.socket.try_send_to(transmit.contents, transmit.destination) {
            Ok(_) => return Ok(()),

            Err(err) if err.kind() == ErrorKind::WouldBlock => {
                todo!()
            },

            Err(err) => Err(err),
        }
    }

    fn poll_recv(
        &self,
        cx: &mut std::task::Context,
        bufs: &mut [std::io::IoSliceMut<'_>],
        meta: &mut [quinn::udp::RecvMeta],
    ) -> Poll<std::io::Result<usize>> {
        ready!(self.socket.poll_recv_ready(cx))?;

        if let Ok(res) = self.socket.try_io(Interest::READABLE, || {
            self.state.recv((&self.socket).into(), bufs, meta)
        }) {
            return Poll::Ready(Ok(res))
        }

        unreachable!()
    }

    fn local_addr(&self) -> std::io::Result<std::net::SocketAddr> {
        self.socket.local_addr()
    }

    fn may_fragment(&self) -> bool {
        self.state.may_fragment()
    }

    fn max_transmit_segments(&self) -> usize {
        self.state.max_gso_segments()
    }

    fn max_receive_segments(&self) -> usize {
        self.state.gro_segments()
    }
}

/*
    The following code is basically copied verbatim from Quinn's internals, repurposed to work with Bevy.
    https://github.com/quinn-rs/quinn/blob/cf445befd313fa55b442581ff9e9b368850e539a/quinn/src/runtime.rs
    Licensed under the MIT OR Apache-2.0 license, depending on what license was chosen by the user for this crate
    Quinn's internals also have far better documentation, so if you want to know what's happening here, go to Quinn
*/

pin_project_lite::pin_project! {
    struct SocketPoller<M, F> {
        m: M,
        #[pin]
        f: Option<F>,
    }   
}

impl<M, F> SocketPoller<M, F> {
    fn new(m: M) -> Self {
        Self { m, f: None }
    }
}

impl<M, F> Debug for SocketPoller<M, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SocketPoller").finish_non_exhaustive()
    }
}

impl<M, F> UdpPoller for SocketPoller<M, F>
where
    M: Fn() -> F + Send + Sync + 'static,
    F: Future<Output = std::io::Result<()>> + Send + Sync + 'static,
{
    fn poll_writable(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context,
    ) -> Poll<std::io::Result<()>> {
        let mut this = self.project();

        if this.f.is_none() {
            this.f.set(Some((this.m)()))
        }

        let result = this.f
            .as_mut()
            .as_pin_mut()
            .unwrap()
            .poll(cx);

        if result.is_ready() {
            this.f.set(None);
        }

        return result;
    }
}