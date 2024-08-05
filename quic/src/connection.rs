use std::{collections::VecDeque, time::{Duration, Instant}};
use bevy_stardust::prelude::*;
use crate::{datagrams::{IncomingDatagrams, OutgoingDatagrams}, ConnectionEvent, ConnectionEventQueue, IncomingStreams, OutgoingStreams, StreamEvent};

/// The core state machine type, representing one QUIC connection.
pub struct Connection {
    last: Instant,

    pub(crate) shared: ConnectionShared,

    pub(crate) incoming_streams: IncomingStreams,
    pub(crate) incoming_datagrams: IncomingDatagrams,

    pub(crate) outgoing_streams: OutgoingStreams,
    pub(crate) outgoing_datagrams: OutgoingDatagrams,
}

impl Connection {
    /// Creates a new [`Connection`] instance.
    pub fn new() -> Self {
        Self {
            last: Instant::now(),

            shared: ConnectionShared {
                heat: Heat::new(),
                event_queue: ConnectionEventQueue::new(),
                recv_queue: VecDeque::new(),
            },

            incoming_streams: IncomingStreams::new(),
            incoming_datagrams: IncomingDatagrams::new(),

            outgoing_streams: OutgoingStreams::new(),
            outgoing_datagrams: OutgoingDatagrams::new(),
        }
    }

    /// Handle timeouts.
    pub fn handle_timeout(&mut self, now: Instant) {
        self.shared.heat.diff(now.duration_since(self.last));
        self.last = now;

        if self.shared.heat.is_overheated() {
            self.shared.event_queue.push(crate::ConnectionEvent::Overheated);
        }
    }

    /// Returns the next event in the queue.
    /// 
    /// Before using this method, call [`handle_timeout`](Self::handle_timeout).
    pub fn poll(&mut self) -> Option<ConnectionEvent> {
        self.shared.event_queue.pop()
    }

    /// Returns the oldest message that has been received.
    pub fn poll_messages(&mut self) -> Option<ChannelMessage> {
        self.shared.recv_queue.pop_front()
    }
}

pub(crate) struct ConnectionShared {
    pub(crate) heat: Heat,
    pub(crate) event_queue: ConnectionEventQueue,
    pub(crate) recv_queue: VecDeque<ChannelMessage>,
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