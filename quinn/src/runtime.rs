use bevy::tasks::AsyncComputeTaskPool;
use quinn::Runtime;

#[derive(Debug)]
pub(crate) struct BevyRuntime;

impl Runtime for BevyRuntime {
    fn new_timer(&self, i: std::time::Instant) -> std::pin::Pin<Box<dyn quinn::AsyncTimer>> {
        todo!()
    }

    fn spawn(&self, future: std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>) {
        AsyncComputeTaskPool::get().spawn(future);
    }

    fn wrap_udp_socket(&self, t: std::net::UdpSocket) -> std::io::Result<std::sync::Arc<dyn quinn::AsyncUdpSocket>> {
        todo!()
    }
}