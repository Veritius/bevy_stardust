mod systems;

pub(crate) use systems::*;

use bevy_stardust::messages::{ChannelId, ChannelMessage};
use bevy::{prelude::*, utils::hashbrown::HashMap};
use quinn_proto::{Connection, ConnectionHandle, ConnectionStats, StreamId};
use crate::{datagrams::{DatagramDesequencer, DatagramSequencer}, streams::{Recv as StRecv, Send as StSend}};

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
    desequencers: HashMap<ChannelId, DatagramDesequencer>,

    channels: HashMap<ChannelId, StreamId>,
    senders: HashMap<StreamId, Box<StSend>>,
    sequencers: HashMap<ChannelId, DatagramSequencer>,

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
            desequencers: HashMap::new(),

            channels: HashMap::new(),
            senders: HashMap::new(),
            sequencers: HashMap::default(),

            pending: Vec::new(),
        }
    }

    /// Returns the full collection of statistics for the connection.
    pub fn stats(&self) -> ConnectionStats {
        self.inner.stats()
    }
}