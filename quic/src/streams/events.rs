use bevy_stardust::prelude::*;
use super::StreamId;

/// A stream-related event.
pub enum StreamEvent {
    /// Send a chunk of data over a stream.
    Transmit {
        /// The stream to send over.
        id: StreamId,

        /// The chunk of data to send.
        chunk: Bytes,
    },

    /// Reset the stream.
    Reset {
        /// The stream to reset.
        id: StreamId,
    },
}