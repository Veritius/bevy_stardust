use bevy::{ecs::component::{ComponentHooks, StorageType}, prelude::*};
use crate::{datagrams::{ChannelDatagrams, IncomingDatagrams, OutgoingDatagrams}, streams::{ChannelStreams, IncomingStreams, OutgoingStreams}, EndpointShared};
use crate::backend::QuicBackend;

/// Shared connection state.
/// 
/// All connections 'belong' to an [`Endpoint`](crate::Endpoint), which they use for I/O.
pub struct ConnectionShared {
    pub(crate) owning_endpoint: Entity,

    pub(crate) incoming_streams: IncomingStreams,
    pub(crate) outgoing_streams: OutgoingStreams,
    pub(crate) channel_streams: ChannelStreams,

    pub(crate) incoming_datagrams: IncomingDatagrams,
    pub(crate) outgoing_datagrams: OutgoingDatagrams,
    pub(crate) channel_datagrams: ChannelDatagrams,
}

impl Component for ConnectionShared {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_remove(|mut world, entity, _| {
            // Get the connection component from the World
            let connection = world.get::<Self>(entity).unwrap();

            // let is_fully_closed = {
            //     #[cfg(feature="quiche")]
            //     connection.quiche.is_closed()
            // };

            // if !is_fully_closed {
            //     warn!("The connection associated with {entity} was dropped when not fully closed");
            // }

            // Try to get the endpoint
            // This may be Err if the endpoint is despawned before the connection
            if let Some(mut endpoint) = world.get_mut::<EndpointShared>(connection.owning_endpoint) {
                // Deregister the connection
                endpoint.connections.deregister(entity);
            }
        });
    }
}

/// Connection state for a connection managed by a [`QuicBackend`] implementor.
pub trait ConnectionState
where
    Self: Send + Sync,
{
    /// The [`QuicBackend`] implementation that manages this connection.
    type Backend: QuicBackend;

    /// Returns `true` if the connection is fully closed and drained,
    /// and that dropping it is guaranteed to not cause data loss.
    fn is_closed(&self) -> bool;
}

#[derive(Component)]
pub struct Connection<State: ConnectionState> {
    state: State,
}