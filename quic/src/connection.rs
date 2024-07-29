use std::collections::BTreeMap;
use bevy_stardust::prelude::ChannelId;
use crate::{datagrams::{IncomingDatagramSequence, OutgoingDatagramSequence}, ConnectionEventIter, ConnectionEventQueue, IncomingStream, OutgoingStreamsState, RecvStreamId, SendStreamId};

/// The core state machine type, representing one QUIC connection.
pub struct Connection {
    pub(crate) events: ConnectionEventQueue,

    pub(crate) incoming_streams: BTreeMap<RecvStreamId, IncomingStream>,
    pub(crate) incoming_datagram_channel_sequences: BTreeMap<ChannelId, IncomingDatagramSequence>,

    pub(crate) outgoing_streams: OutgoingStreamsState,
    pub(crate) outgoing_channel_stream_ids: BTreeMap<ChannelId, SendStreamId>,
    pub(crate) outgoing_datagram_channel_sequences: BTreeMap<ChannelId, OutgoingDatagramSequence>,
}

impl Connection {
    /// Creates a new [`Connection`] instance.
    pub fn new() -> Self {
        Self {
            events: ConnectionEventQueue::new(),

            incoming_streams: BTreeMap::new(),
            incoming_datagram_channel_sequences: BTreeMap::new(),

            outgoing_streams: OutgoingStreamsState::new(),
            outgoing_channel_stream_ids: BTreeMap::new(),
            outgoing_datagram_channel_sequences: BTreeMap::new(),
        }
    }

    /// Returns an iterator over the event queue.
    pub fn poll(&mut self) -> ConnectionEventIter {
        ConnectionEventIter::new(&mut self.events)
    }
}

pub(crate) struct Heat {
    resource: u32,
    strange: u32,
}