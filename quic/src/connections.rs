use std::{sync::Exclusive, time::Instant};
use bytes::*;
use quinn_proto::*;
use bevy_ecs::prelude::*;

/// A QUIC connection.
/// 
/// This component will be present even during a handshake.
/// Once the handshake is complete, the `NetworkPeer` component will be added.
#[derive(Component)]
pub struct QuicConnection {
    pub(crate) endpoint: Entity,
    pub(crate) inner: Exclusive<Connection>,
}

impl QuicConnection {
    pub(crate) fn new(
        endpoint: Entity,
        connection: Connection
    ) -> Self {
        Self {
            endpoint,
            inner: Exclusive::new(connection),
        }
    }

    /// Returns the entity ID of the endpoint performing IO for this connection.
    pub fn endpoint(&self) -> Entity {
        self.endpoint
    }

    /// Closes the connection.
    pub fn close(&mut self, reason: Bytes) {
        self.inner.get_mut().close(Instant::now(), VarInt::default(), reason)
    }
}