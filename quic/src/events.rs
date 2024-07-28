use crate::StreamEvent;

/// An event sent by the connection state machine.
pub enum ConnectionEvent {
    /// A stream event.
    StreamEvent(StreamEvent),
}