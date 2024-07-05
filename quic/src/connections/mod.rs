mod systems;

pub(crate) use systems::*;

use bevy_stardust::messages::{ChannelId, ChannelMessage};
use bevy::{prelude::*, utils::hashbrown::HashMap};
use quinn_proto::{Connection, ConnectionHandle, ConnectionStats, StreamId};
use crate::streams::{Send as StSend, Recv as StRecv};

/// A QUIC connection, attached to an endpoint.
/// 
/// # Safety
/// This component must always stay in the same [`World`] as it was created in.
/// Being put into another `World` will lead to undefined behavior.
#[derive(Component)]
pub struct QuicConnection {
    pub(crate) owner: Entity,
    pub(crate) handle: ConnectionHandle,
    pub(crate) inner: Box<Connection>,

    readers: HashMap<StreamId, Box<StRecv>>,

    channels: HashMap<ChannelId, StreamId>,
    senders: HashMap<StreamId, Box<StSend>>,

    pending: Vec<ChannelMessage>,
}

impl QuicConnection {
    pub(crate) fn new(
        owner: Entity,
        handle: ConnectionHandle,
        inner: Box<Connection>,
    ) -> Self {
        Self {
            owner,
            handle,
            inner,

            readers: HashMap::new(),

            channels: HashMap::new(),
            senders: HashMap::new(),

            pending: Vec::new(),
        }
    }

    /// Returns the full collection of statistics for the connection.
    pub fn stats(&self) -> ConnectionStats {
        self.inner.stats()
    }
}