mod datagrams;
mod streams;

use bevy::prelude::*;
use datagrams::{ChannelDatagrams, IncomingDatagrams, OutgoingDatagrams};
use streams::{ChannelStreams, IncomingStreams, OutgoingStreams};
use crate::backend::QuicBackend;

pub use datagrams::DatagramManager;
pub use streams::{StreamManager, SendStream, StreamSendOutcome, RecvStream, StreamRecvOutcome};
pub use streams::StreamId;

/// Shared connection state.
/// 
/// All connections 'belong' to an [`Endpoint`](crate::Endpoint), which they use for I/O.
#[derive(Component)]
pub struct Connection {
    pub(crate) owning_endpoint: Entity,

    incoming_streams: IncomingStreams,
    outgoing_streams: OutgoingStreams,
    channel_streams: ChannelStreams,

    incoming_datagrams: IncomingDatagrams,
    outgoing_datagrams: OutgoingDatagrams,
    channel_datagrams: ChannelDatagrams,
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
pub(crate) struct ConnectionStateData<State: ConnectionState> {
    state: State,
}

impl<State: ConnectionState> ConnectionStateData<State> {
    pub(crate) fn inner(&mut self) -> &mut State {
        &mut self.state
    }
}