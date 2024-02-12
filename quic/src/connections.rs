use std::{sync::Exclusive, time::Instant};
use bytes::*;
use quinn_proto::*;
use bevy::prelude::*;
use bevy_stardust::prelude::*;

#[derive(Bundle)]
pub(crate) struct QuicConnectionBundle {
    pub peer_comp: NetworkPeer,
    pub quic_comp: QuicConnection,
}

/// An active QUIC connection.
#[derive(Component)]
pub struct QuicConnection {
    pub(crate) inner: Exclusive<Connection>,
}

impl QuicConnection {
    /// Closes the connection.
    pub fn close(&mut self, reason: Bytes) {
        self.inner.get_mut().close(Instant::now(), VarInt::default(), reason)
    }
}