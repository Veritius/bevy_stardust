mod datagrams;
mod streams;
mod systems;

use datagrams::*;
use streams::*;

pub(crate) use systems::*;

use bevy::{ecs::component::{ComponentHooks, StorageType}, prelude::*};
use quinn_proto::{Connection, ConnectionHandle, ConnectionStats, EndpointEvent};
use crate::QuicEndpoint;

/// A QUIC connection, attached to an endpoint.
/// 
/// # Safety
/// This component must always stay in the same [`World`] as it was created in.
/// Being put into another `World` will lead to undefined behavior.
pub struct QuicConnection {
    pub(crate) owner: Entity,
    pub(crate) handle: ConnectionHandle,
    pub(crate) quinn: Box<Connection>,

    incoming_streams: IncomingStreams,
    outgoing_streams: OutgoingStreams,
    channel_streams: ChannelStreams,

    incoming_datagrams: IncomingDatagrams,
    outgoing_datagrams: OutgoingDatagrams,
    channel_datagrams: ChannelDatagrams,
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
            quinn: inner,

            incoming_streams: IncomingStreams::new(),
        outgoing_streams: OutgoingStreams::new(),
        channel_streams: ChannelStreams::new(),

        incoming_datagrams: IncomingDatagrams::new(),
        outgoing_datagrams: OutgoingDatagrams::new(),
        channel_datagrams: ChannelDatagrams::new(),
        }
    }

    /// Returns the full collection of statistics for the connection.
    pub fn stats(&self) -> ConnectionStats {
        self.quinn.stats()
    }
}

impl Component for QuicConnection {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_remove(|mut world, entity, _| {
            // Check the component isn't drained
            let connection = world.get::<Self>(entity).unwrap();
            if connection.quinn.is_drained() { return }

            // Check if the component isn't closed
            if !connection.quinn.is_closed() {
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