use std::task::{ready, Poll};
use bevy::tasks::{AsyncComputeTaskPool, ComputeTaskPool, IoTaskPool};

/// Schedules tasks for processing on Bevy task pools.
#[derive(Debug)]
pub(crate) enum BevyAsyncExecutor {
    IoCompute,
    SyncCompute,
    AsyncCompute,
}

impl quinn::Runtime for BevyAsyncExecutor {
    fn new_timer(&self, i: std::time::Instant) -> std::pin::Pin<Box<dyn quinn::AsyncTimer>> {
        Box::pin(AsyncTimer(async_io::Timer::at(i))) as _
    }

    fn spawn(&self, future: std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>) {
        match self {
            Self::IoCompute => {
                let pool = IoTaskPool::get();
                pool.spawn(future).detach();
            }
            Self::SyncCompute => {
                let pool = ComputeTaskPool::get();
                pool.spawn(future).detach();
            }
            Self::AsyncCompute => {
                let pool = AsyncComputeTaskPool::get();
                pool.spawn(future).detach();
            }
        }
    }

    fn wrap_udp_socket(&self, t: std::net::UdpSocket) -> std::io::Result<Box<dyn quinn::AsyncUdpSocket>> {
        Ok(Box::new(AsyncUdpSocket {
            io: async_io::Async::new(t)?,
            inner: quinn_udp::UdpSocketState::new(),
        }))
    }
}

#[derive(Debug)]
pub(crate) struct AsyncTimer(async_io::Timer);

impl quinn::AsyncTimer for AsyncTimer {
    fn reset(mut self: std::pin::Pin<&mut Self>, i: std::time::Instant) {
        self.0 = async_io::Timer::at(i);
    }

    fn poll(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context) -> std::task::Poll<()> {
        let timer = std::pin::Pin::new(&mut self.0);
        <async_io::Timer as std::future::Future>::poll(timer, cx).map(|_| ())
    }
}

#[derive(Debug)]
struct AsyncUdpSocket {
    io: async_io::Async<std::net::UdpSocket>,
    inner: quinn_udp::UdpSocketState,
}

impl quinn::AsyncUdpSocket for AsyncUdpSocket {
    fn poll_send(
        &self,
        state: &quinn_udp::UdpState,
        cx: &mut std::task::Context,
        transmits: &[quinn_udp::Transmit],
    ) -> Poll<Result<usize, std::io::Error>> {
        loop {
            ready!(self.io.poll_writable(cx))?;
            if let Ok(res) = self.inner.send((&self.io).into(), state, transmits) {
                return Poll::Ready(Ok(res));
            }
        }
    }

    fn poll_recv(
        &self,
        cx: &mut std::task::Context,
        bufs: &mut [std::io::IoSliceMut<'_>],
        meta: &mut [quinn_udp::RecvMeta],
    ) -> std::task::Poll<std::io::Result<usize>> {
        loop {
            ready!(self.io.poll_readable(cx))?;
            if let Ok(res) = self.inner.recv((&self.io).into(), bufs, meta) {
                return Poll::Ready(Ok(res));
            }
        }
    }

    fn local_addr(&self) -> std::io::Result<std::net::SocketAddr> {
        self.io.as_ref().local_addr()
    }
}