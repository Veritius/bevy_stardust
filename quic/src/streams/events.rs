use bevy_stardust::prelude::*;
use super::{RecvStreamId, SendStreamId};

/// A stream-related event.
pub enum StreamEvent {
    /// Send a chunk of data over a stream.
    Transmit {
        /// The stream to send over.
        id: SendStreamId,

        /// The chunk of data to send.
        chunk: Bytes,
    },

    /// Set the priority of a stream.
    SetPriority {
        /// The stream which should have its priority changed.
        id: SendStreamId,

        /// The priority value.
        priority: u32,
    },

    /// Reset a stream.
    Reset {
        /// The stream to reset.
        id: SendStreamId,
    },

    /// Finish a stream.
    Finish {
        /// The stream to finish.
        id: SendStreamId,
    },

    /// Stop a stream.
    Stop {
        /// The stream to stop.
        id: RecvStreamId,
    },
}