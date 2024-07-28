mod events;
mod framing;
mod header;
mod incoming;

use crate::Connection;

pub(crate) use incoming::IncomingStream;

pub use events::StreamEvent;

impl Connection {
    /// Call when a new incoming stream is opened.
    pub fn stream_opened(&mut self, stream: RecvStreamId) {
        self.incoming_streams.insert(stream, IncomingStream::new());
    }

    /// Call when a stream is reset.
    pub fn stream_reset(&mut self, stream: RecvStreamId) {
        self.incoming_streams.remove(&stream);
    }

    /// Call when a stream is finished.
    pub fn stream_finished(&mut self, stream: RecvStreamId) {
        self.incoming_streams.remove(&stream);
    }

    /// Call when a stream is stopped.
    pub fn stream_stopped(&mut self, stream: SendStreamId) {
        todo!()
    }
}

/// A stream identifier for an **outgoing** (sending) QUIC stream.
/// 
/// Generated by the state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SendStreamId(pub u64);

/// A stream identifier for an **incoming** (receiving) QUIC stream.
/// 
/// Generated by the QUIC implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RecvStreamId(pub u64);