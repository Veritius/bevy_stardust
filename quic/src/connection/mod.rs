mod datagrams;
mod events;
mod state;
mod streams;

use bevy::{ecs::component::{ComponentHooks, StorageType}, prelude::*};
use datagrams::{ChannelDatagrams, IncomingDatagrams, OutgoingDatagrams};
use streams::{ChannelStreams, IncomingStreams, OutgoingStreams};
use crate::Endpoint;

/// A QUIC connection.
/// 
/// All connections 'belong' to an [`Endpoint`](crate::Endpoint), which they use for I/O.
pub struct Connection {

}

impl Component for Connection {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_remove(|mut world, entity, _| {
            // Get the connection component from the World
            let connection = world.get::<Self>(entity).unwrap();

            let is_fully_closed: bool = todo!(); //{
            //     #[cfg(feature="quiche")]
            //     connection.quiche.is_closed()
            // };

            if !is_fully_closed {
                warn!("The connection associated with {entity} was dropped when not fully closed");
            }

            // Try to get the endpoint
            // This may be Err if the endpoint is despawned before the connection
            // if let Some(mut endpoint) = world.get_mut::<Endpoint>(connection.endpoint) {
            //     // Deregister the connection
            //     endpoint.connections.deregister(entity);
            // }
        });
    }
}