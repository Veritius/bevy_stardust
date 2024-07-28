use crate::endpoint::{UdpSocketRecv, UdpSocketSend};
use super::QuicBackend;

/// An endpoint associated with a [`Backend`](crate::backend::Backend) implementation.
pub trait EndpointState
where
    Self: Send + Sync,
{
    /// The [`QuicBackend`] implementation that manages this endpoint.
    type Backend: QuicBackend;

    fn recv<'a>(
        &'a mut self,
        backend: &'a Self::Backend,
        socket: UdpSocketRecv<'a>,
        connections: Connections<'a>,
    );

    fn send<'a>(
        &'a mut self,
        backend: &'a Self::Backend,
        socket: UdpSocketSend<'a>,
        connections: Connections<'a>,
    );
}

pub struct Connections<'a> {
    phantom: std::marker::PhantomData<&'a ()>,
}