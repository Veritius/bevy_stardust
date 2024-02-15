use std::net::{SocketAddr, UdpSocket};
use bevy_ecs::prelude::*;
use bytes::Bytes;
use smallvec::SmallVec;

/// An endpoint, which is used for I/O.
/// 
/// Removing this component will not inform clients, and they will eventually time out.
/// Any information from the client that hasn't been received will never be received.
/// Instead of removing this component, consider using the [`close`](Self::close) method.
#[derive(Component)]
pub struct Endpoint(pub(crate) EndpointInner);

impl Endpoint {
    /// Marks the endpoint for closure.
    /// This will inform all clients of the disconnection along with the `reason` if present,
    /// and waits for data exchange to stop. This is the best solution for most use cases.
    /// 
    /// If `hard` is set to `true`, the endpoint will be closed as soon as possible.
    /// A message will be sent to inform clients but nothing will be done to ensure its arrival.
    /// Messages from the client that haven't been received will never be received.
    pub fn close(&mut self, hard: bool, reason: Option<Bytes>) {
        todo!()
    }

    /// Sets whether or not to accept new incoming connections.
    pub fn set_listen(&mut self, listen: bool) {
        self.0.listening = listen;
    }

    /// Returns the local IP address that this Endpoint is bound to.
    pub fn local_addr(&self) -> SocketAddr {
        // Unwrapping is fine because an Endpoint always has a local address
        self.0.socket.local_addr().unwrap()
    }
}

pub(crate) struct EndpointInner {
    pub socket: UdpSocket,
    pub listening: bool,
    pub connections: SmallVec::<[Entity; 16]>,
}