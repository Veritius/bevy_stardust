mod datagrams;
mod streams;

use bevy::prelude::*;
use datagrams::{ChannelDatagrams, IncomingDatagrams, OutgoingDatagrams};
use streams::{ChannelStreams, IncomingStreams, OutgoingStreams};
use crate::backend::ConnectionState;

pub use crate::backend::{StreamManager, SendStream, StreamSendOutcome, RecvStream, StreamRecvOutcome};

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

#[derive(Component)]
pub(crate) struct ConnectionStateData<State: ConnectionState> {
    state: State,
}

impl<State: ConnectionState> ConnectionStateData<State> {
    pub(crate) fn inner(&mut self) -> &mut State {
        &mut self.state
    }
}