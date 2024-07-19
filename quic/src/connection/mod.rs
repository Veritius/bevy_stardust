mod streams;
mod datagrams;

use bevy::{ecs::component::{ComponentHooks, StorageType}, prelude::*};
use datagrams::{ChannelDatagrams, IncomingDatagrams, OutgoingDatagrams};
use streams::{ChannelStreams, IncomingStreams, OutgoingStreams};
use crate::EndpointShared;
use crate::backend::QuicBackend;

pub use datagrams::DatagramManager;
pub use streams::{StreamManager, SendStream, StreamSendOutcome, RecvStream, StreamRecvOutcome};
pub use streams::StreamId;

/// Shared connection state.
/// 
/// All connections 'belong' to an [`Endpoint`](crate::Endpoint), which they use for I/O.
pub struct ConnectionShared {
    pub(crate) owning_endpoint: Entity,

    incoming_streams: IncomingStreams,
    outgoing_streams: OutgoingStreams,
    channel_streams: ChannelStreams,

    incoming_datagrams: IncomingDatagrams,
    outgoing_datagrams: OutgoingDatagrams,
    channel_datagrams: ChannelDatagrams,
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

    /// A handle returned by the [`datagrams`](ConnectionState::datagrams) method.
    type Datagrams<'a>: DatagramManager where Self: 'a;

    /// A handle returned by the [`streams`](ConnectionState::streams) method.
    type Streams<'a>: StreamManager where Self: 'a;

    /// Returns `true` if the connection is fully closed and drained,
    /// and that dropping it is guaranteed to not cause data loss.
    fn is_closed(&self) -> bool;

    /// Get a handle to a `DatagramManager` for this connection.
    fn datagrams(&mut self) -> Self::Datagrams<'_>;

    /// Get a handle to a `StreamManager` for this connection.
    fn streams(&mut self) -> Self::Streams<'_>;
}

#[derive(Component)]
pub struct Connection<State: ConnectionState> {
    state: State,
}