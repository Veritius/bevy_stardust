use std::{collections::{HashMap, VecDeque}, sync::Exclusive, time::Instant};
use bevy_stardust::channels::id::ChannelId;
use bytes::*;
use quinn_proto::*;
use bevy_ecs::prelude::*;
use crate::streams::{IncomingBufferedStreamData, OutgoingBufferedStreamData};

/// A QUIC connection.
/// 
/// This component will be present even during a handshake.
/// Once the handshake is complete, the `NetworkPeer` component will be added.
#[derive(Component)]
pub struct QuicConnection {
    pub(crate) endpoint: Entity,
    pub(crate) handle: ConnectionHandle,
    pub(crate) inner: Exclusive<Connection>,

    pub(crate) transient_send_streams: VecDeque<OutgoingBufferedStreamData>,
    pub(crate) persistent_send_streams: HashMap<ChannelId, OutgoingBufferedStreamData>,
    pub(crate) recv_streams: HashMap<StreamId, IncomingBufferedStreamData>,

    pub(crate) force_despawn: bool,
}

impl QuicConnection {
    pub(crate) fn new(
        endpoint: Entity,
        handle: ConnectionHandle,
        connection: Connection
    ) -> Self {
        Self {
            endpoint,
            handle,
            inner: Exclusive::new(connection),
            transient_send_streams: VecDeque::default(),
            persistent_send_streams: HashMap::default(),
            recv_streams: HashMap::default(),
            force_despawn: false,
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