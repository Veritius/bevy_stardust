use std::collections::VecDeque;
use hashbrown::HashMap;
use crate::{ConnectionEvent, IncomingStream, StreamId};

/// The core state machine type, representing one QUIC connection.
pub struct Connection {
    events: VecDeque<ConnectionEvent>,

    incoming_streams: HashMap<StreamId, IncomingStream>,
}

impl Connection {
    /// Creates a new [`Connection`] instance.
    pub fn new() -> Self {
        Self {
            events: VecDeque::new(),

            incoming_streams: HashMap::new(),
        }
    }

    /// Returns an event if one has occurred.
    pub fn poll_events(&mut self) -> Option<ConnectionEvent> {
        self.events.pop_front()
    }

    pub(crate) fn queue_event(&mut self, event: ConnectionEvent) {
        self.events.push_back(event)
    }
}