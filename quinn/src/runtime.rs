use std::{fmt::Debug, future::Future, io::ErrorKind, net::UdpSocket, sync::Arc, task::Poll, time::Instant};
use bevy::tasks::{futures_lite::ready, ComputeTaskPool};
use quinn::{udp::UdpSocketState, AsyncTimer, AsyncUdpSocket, Runtime, UdpPoller};

#[derive(Debug)]
pub(crate) struct BevyRuntime;

impl Runtime for BevyRuntime {
    fn new_timer(&self, i: std::time::Instant) -> std::pin::Pin<Box<dyn quinn::AsyncTimer>> {
        Box::pin(Timer(todo!()))
    }

    fn spawn(&self, future: std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>) {
        ComputeTaskPool::get().spawn(future);
    }

    fn wrap_udp_socket(&self, t: std::net::UdpSocket) -> std::io::Result<std::sync::Arc<dyn quinn::AsyncUdpSocket>> {
        t.set_nonblocking(true)?;

        return Ok(Arc::new(Socket {
            state: UdpSocketState::new((&t).into())?,
            socket: t,
        }));
    }
}

#[derive(Debug)]
pub(crate) struct Timer(());

impl AsyncTimer for Timer {
    fn reset(self: std::pin::Pin<&mut Self>, i: std::time::Instant) {
        todo!()
    }

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context) -> std::task::Poll<()> {
        todo!()
    }
}

#[derive(Debug)]
pub(crate) struct Socket {
    state: UdpSocketState,
    socket: UdpSocket,
}

impl AsyncUdpSocket for Socket {
    fn create_io_poller(self: std::sync::Arc<Self>) -> std::pin::Pin<Box<dyn quinn::UdpPoller>> {
        todo!()
    }

    fn try_send(&self, transmit: &quinn::udp::Transmit) -> std::io::Result<()> {
        todo!()
    }

    fn poll_recv(
        &self,
        cx: &mut std::task::Context,
        bufs: &mut [std::io::IoSliceMut<'_>],
        meta: &mut [quinn::udp::RecvMeta],
    ) -> Poll<std::io::Result<usize>> {
        todo!()
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