use std::time::{Duration, Instant};
use crate::{ConnectionEvent, ConnectionEventQueue, IncomingStreams, MessageSequenceMap, OutgoingStreams};

/// The core state machine type, representing one QUIC connection.
pub struct Connection {
    last: Instant,

    pub(crate) shared: ConnectionShared,

    pub(crate) outgoing_streams: OutgoingStreams,
    pub(crate) incoming_streams: IncomingStreams,

    pub(crate) message_sequences: MessageSequenceMap,
}

impl Connection {
    /// Creates a new [`Connection`] instance.
    pub fn new() -> Self {
        Self {
            last: Instant::now(),

            shared: ConnectionShared {
                heat: Heat::new(),
                events: ConnectionEventQueue::new(),
            },

            outgoing_streams: OutgoingStreams::new(),
            incoming_streams: IncomingStreams::new(),

            message_sequences: MessageSequenceMap::new(),
        }
    }

    /// Handle timeouts.
    pub fn handle_timeout(&mut self, now: Instant) {
        self.shared.heat.diff(now.duration_since(self.last));
        self.last = now;

        if self.shared.heat.is_overheated() {
            self.shared.events.push(crate::ConnectionEvent::Overheated);
        }
    }

    /// Returns the next event in the queue.
    /// 
    /// Before using this method, call [`handle_timeout`](Self::handle_timeout).
    pub fn poll(&mut self) -> Option<ConnectionEvent> {
        self.shared.events.pop()
    }
}

pub(crate) struct ConnectionShared {
    pub(crate) heat: Heat,
    pub(crate) events: ConnectionEventQueue,
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