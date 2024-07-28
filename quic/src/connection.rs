use std::collections::{BTreeMap, VecDeque};
use bevy_stardust::prelude::ChannelId;
use crate::{ConnectionEvent, IncomingDatagramSequence, IncomingStream, OutgoingDatagramSequence, RecvStreamId, SendStreamId};

/// The core state machine type, representing one QUIC connection.
pub struct Connection {
    pub(crate) events: VecDeque<ConnectionEvent>,

    pub(crate) incoming_streams: BTreeMap<RecvStreamId, IncomingStream>,
    pub(crate) incoming_datagram_channel_sequences: BTreeMap<ChannelId, IncomingDatagramSequence>,

    pub(crate) outgoing_channel_stream_ids: BTreeMap<ChannelId, SendStreamId>,
    pub(crate) outgoing_datagram_channel_sequences: BTreeMap<ChannelId, OutgoingDatagramSequence>,
}

impl Connection {
    /// Creates a new [`Connection`] instance.
    pub fn new() -> Self {
        Self {
            events: VecDeque::new(),

            incoming_streams: BTreeMap::new(),
            incoming_datagram_channel_sequences: BTreeMap::new(),

            outgoing_channel_stream_ids: BTreeMap::new(),
            outgoing_datagram_channel_sequences: BTreeMap::new(),
        }
    }

    /// Returns an event if one has occurred.
    pub fn poll(&mut self) -> Option<ConnectionEvent> {
        self.events.pop_front()
    }
}