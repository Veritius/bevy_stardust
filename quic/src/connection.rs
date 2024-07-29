use std::{collections::BTreeMap, time::{Duration, Instant}};
use bevy_stardust::prelude::ChannelId;
use crate::{datagrams::{IncomingDatagramSequence, OutgoingDatagramSequence}, ConnectionEventIter, ConnectionEventQueue, IncomingStream, OutgoingStreamsState, RecvStreamId, SendStreamId};

/// The core state machine type, representing one QUIC connection.
pub struct Connection {
    last: Instant,

    pub(crate) heat: Heat,
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
            last: Instant::now(),

            heat: Heat::new(),
            events: ConnectionEventQueue::new(),

            incoming_streams: BTreeMap::new(),
            incoming_datagram_channel_sequences: BTreeMap::new(),

            outgoing_streams: OutgoingStreamsState::new(),
            outgoing_channel_stream_ids: BTreeMap::new(),
            outgoing_datagram_channel_sequences: BTreeMap::new(),
        }
    }

    /// Returns an iterator over the event queue.
    pub fn poll(&mut self, now: Instant) -> ConnectionEventIter {
        self.heat.diff(now.duration_since(self.last));
        self.last = now;

        if self.heat.is_overheated() {
            self.events = ConnectionEventQueue::with_capacity(1);
            self.events.push(crate::ConnectionEvent::Overheated);
        }

        ConnectionEventIter::new(&mut self.events)
    }
}

#[derive(Debug)]
pub(crate) struct Heat {
    value: u32,
}

impl Heat {
    const LIMIT: u32 = 65535;
    const COOLING: u32 = 1024;

    fn new() -> Self {
        Self {
            value: 0,
        }
    }

    fn diff(&mut self, dur: Duration) {
        let cooling = dur.as_millis() as u32 / Self::COOLING;
        self.value -= cooling;
    }

    pub fn is_overheated(&self) -> bool {
        self.value >= Self::LIMIT
    }

    pub fn increase(&mut self, amt: u32) {
        self.value = self.value.saturating_add(amt);
    }
}