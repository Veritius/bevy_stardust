use std::collections::VecDeque;
use crate::ConnectionEvent;

/// The core state machine type, representing one QUIC connection.
pub struct Connection {
    events: VecDeque<ConnectionEvent>,
}

impl Connection {
    /// Creates a new [`Connection`] instance.
    pub fn new() -> Self {
        Self {
            events: VecDeque::new(),
        }
    }

    /// Returns an event if one has occurred.
    pub fn poll_events(&mut self) -> Option<ConnectionEvent> {
        self.events.pop_front()
    }
}