mod systems;

pub(crate) use systems::*;

use bevy_stardust::messages::{ChannelId, ChannelMessage};
use bevy::{ecs::component::{ComponentHooks, StorageType}, prelude::*, utils::hashbrown::HashMap};
use quinn_proto::{Connection, ConnectionHandle, ConnectionStats, EndpointEvent, StreamId};
use crate::{datagrams::{DatagramDesequencer, DatagramSequencer}, streams::{Recv as StRecv, Send as StSend}, QuicEndpoint};

/// A QUIC connection, attached to an endpoint.
/// 
/// # Safety
/// This component must always stay in the same [`World`] as it was created in.
/// Being put into another `World` will lead to undefined behavior.
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

impl Component for QuicConnection {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_remove(|mut world, entity, _| {
            // Check the component isn't drained
            let connection = world.get::<Self>(entity).unwrap();
            if connection.inner.is_drained() { return }

            // Check if the component isn't closed
            if !connection.inner.is_closed() {
                warn!(connection=?entity, "A connection was removed without being fully closed");
            }

            // Remove the handle from the endpoint
            let handle = connection.handle;
            if let Some(mut endpoint) = world.get_mut::<QuicEndpoint>(connection.owner) {
                endpoint.entities.remove(&handle);
                endpoint.inner.handle_event(handle, EndpointEvent::drained()); // informs endpoint of drop
            }
        });
    }
}