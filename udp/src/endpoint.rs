use std::net::UdpSocket;
use bevy_ecs::prelude::*;
use bytes::Bytes;
use smallvec::SmallVec;

/// An endpoint, which is used for I/O.
/// 
/// Removing this component will not inform clients, and they will eventually time out.
/// Any information from the client that hasn't been received will never be received.
/// Instead of removing this component, consider using the [`close`](Self::close) method.
#[derive(Component)]
#[cfg_attr(feature="reflect", derive(bevy_reflect::Reflect), reflect(from_reflect = false))]
pub struct Endpoint {
    #[cfg_attr(feature="reflect", reflect(ignore))]
    pub(crate) socket: UdpSocket,

    #[cfg_attr(feature="reflect", reflect(ignore))]
    pub(crate) connections: SmallVec::<[Entity; 16]>,

    /// Whether or not to accept new incoming connections on this endpoint.
    pub listening: bool,
}

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
}