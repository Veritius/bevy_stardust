use std::{collections::{HashMap, VecDeque}, sync::Exclusive, time::Instant};
use bevy_stardust::channels::id::ChannelId;
use bytes::*;
use quinn_proto::*;
use bevy_ecs::prelude::*;
use crate::{reading::IncomingStream, streams::OutgoingBufferedStreamData};

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
    pub(crate) recv_streams: HashMap<StreamId, IncomingStream>,

    pub(crate) connection_state: ConnectionStateData,
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
            connection_state: ConnectionStateData::QuicHandshake,
            force_despawn: false,
        }
    }

    /// Returns the entity ID of the endpoint performing IO for this connection.
    pub fn endpoint(&self) -> Entity {
        self.endpoint
    }

    /// Returns the state of the connection.
    pub fn state(&self) -> ConnectionState {
        self.connection_state.flat()
    }

    /// Closes the connection.
    pub fn close(&mut self, reason: Bytes) {
        self.connection_state = ConnectionStateData::Disconnecting { reason };
    }
}

#[derive(Debug)]
pub(crate) enum ConnectionStateData {
    QuicHandshake,
    GameHandshake {
        passed_version_check: bool,

        #[cfg(feature="hash_check")]
        passed_hash_check: bool,
    },
    Connected,
    Disconnecting {
        reason: Bytes,
    },
    Disconnected,
}

impl ConnectionStateData {
    /// Returns a variant without additional data.
    pub fn flat(&self) -> ConnectionState {
        match self {
            ConnectionStateData::QuicHandshake => ConnectionState::Handshake,

            #[cfg(not(feature="hash_check"))]
            ConnectionStateData::GameHandshake {
                passed_version_check: _,
            } => ConnectionState::Handshake,

            #[cfg(feature="hash_check")]
            ConnectionStateData::GameHandshake {
                passed_version_check: _,
                passed_hash_check: _,
            } => ConnectionState::Handshake,

            ConnectionStateData::Connected => ConnectionState::Connected,

            ConnectionStateData::Disconnecting {
                reason: _
            } => ConnectionState::Disconnecting,

            ConnectionStateData::Disconnected => ConnectionState::Disconnected,
        }
    }
}

// this is exposed, ConnectionStateData isn't
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConnectionState {
    Handshake,
    Connected,
    Disconnecting,
    Disconnected,
}