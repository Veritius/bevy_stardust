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

    /// Set the priority of a stream.
    SetPriority {
        /// The stream which should have its priority changed.
        id: StreamId,

        /// The priority value.
        priority: u32,
    },

    /// Stop a stream.
    Stop {
        /// The stream to stop.
        id: StreamId,
    },

    /// Reset a stream.
    Reset {
        /// The stream to reset.
        id: StreamId,
    },

    /// Finish a stream.
    Finish {
        /// The stream to finish.
        id: StreamId,
    }
}