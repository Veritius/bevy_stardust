use std::{net::UdpSocket, time::Instant};
use bevy::tasks::ComputeTaskPool;
use quinn::{AsyncTimer, AsyncUdpSocket, Runtime};

#[derive(Debug)]
pub(crate) struct BevyRuntime;

impl Runtime for BevyRuntime {
    fn new_timer(&self, i: std::time::Instant) -> std::pin::Pin<Box<dyn quinn::AsyncTimer>> {
        Box::pin(Timer(i))
    }

    fn spawn(&self, future: std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>) {
        ComputeTaskPool::get().spawn(future);
    }

    fn wrap_udp_socket(&self, t: std::net::UdpSocket) -> std::io::Result<std::sync::Arc<dyn quinn::AsyncUdpSocket>> {
        todo!()
    }
}

#[derive(Debug)]
pub(crate) struct Socket {
    socket: UdpSocket,
}

impl AsyncUdpSocket for Socket {
    fn create_io_poller(self: std::sync::Arc<Self>) -> std::pin::Pin<Box<dyn quinn::UdpPoller>> {
        todo!()
    }

    fn try_send(&self, transmit: &quinn::udp::Transmit) -> std::io::Result<()> {
        self.socket.send_to(transmit.contents, transmit.destination).map(|_| ())
    }

    fn poll_recv(
        &self,
        cx: &mut std::task::Context,
        bufs: &mut [std::io::IoSliceMut<'_>],
        meta: &mut [quinn::udp::RecvMeta],
    ) -> std::task::Poll<std::io::Result<usize>> {
        todo!()
    }

    fn local_addr(&self) -> std::io::Result<std::net::SocketAddr> {
        self.socket.local_addr()
    }
}

#[derive(Debug)]
pub(crate) struct Timer(Instant);

impl AsyncTimer for Timer {
    fn reset(mut self: std::pin::Pin<&mut Self>, i: std::time::Instant) {
        self.0 = i;
    }

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context) -> std::task::Poll<()> {
        todo!()
    }
}