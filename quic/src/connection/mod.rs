pub mod datagrams;
pub mod streams;

use bevy::{ecs::component::{ComponentHooks, StorageType}, prelude::*};
use datagrams::{ChannelDatagrams, IncomingDatagrams, OutgoingDatagrams};
use streams::{ChannelStreams, IncomingStreams, OutgoingStreams};

use crate::Endpoint;

/// A QUIC connection.
/// 
/// All connections 'belong' to an [`Endpoint`](crate::Endpoint), which they use for I/O.
#[derive(Reflect)]
#[reflect(from_reflect=false, Component)]
pub struct Connection {
    #[reflect(ignore)]
    pub(crate) endpoint: Entity,

    #[cfg(feature="quiche")]
    #[reflect(ignore)]
    pub(crate) quiche: crate::quiche::QuicheConnection,

    #[reflect(ignore)]
    pub(crate) incoming_streams: IncomingStreams,

    #[reflect(ignore)]
    pub(crate) outgoing_streams: OutgoingStreams,

    #[reflect(ignore)]
    pub(crate) channel_streams: ChannelStreams,

    #[reflect(ignore)]
    pub(crate) incoming_datagrams: IncomingDatagrams,

    #[reflect(ignore)]
    pub(crate) outgoing_datagrams: OutgoingDatagrams,

    #[reflect(ignore)]
    pub(crate) channel_datagrams: ChannelDatagrams,
}

impl Component for Connection {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_remove(|mut world, entity, _| {
            // Get the connection component from the World
            let connection = world.get::<Self>(entity).unwrap();

            let is_fully_closed = {
                #[cfg(feature="quiche")]
                connection.quiche.is_closed()
            };

            if !is_fully_closed {
                warn!("The connection associated with {entity} was dropped when not fully closed");
            }

            // Try to get the endpoint
            // This may be Err if the endpoint is despawned before the connection
            if let Some(mut endpoint) = world.get_mut::<Endpoint>(connection.endpoint) {
                // Deregister the connection
                endpoint.connections.deregister(entity);
            }
        });
    }
}