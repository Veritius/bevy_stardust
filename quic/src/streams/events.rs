use bevy_stardust::prelude::*;
use super::{RecvStreamId, SendStreamId};

/// An event used by the state machine to control QUIC streams.
pub enum TransportStreamEvent {
    /// A new stream was opened.
    Opened {
        /// The ID of the opened stream.
        id: RecvStreamId,
    },

    /// A stream was reset.
    Reset {
        /// The ID of the reset stream.
        id: RecvStreamId,
    },

    /// A stream was finished.
    Finished {
        /// The ID of the finished stream.
        id: RecvStreamId,
    },

    /// A stream was stopped.
    Stopped {
        /// The ID of the stopped stream.
        id: SendStreamId,
    },
}

pub enum ConnectionStreamEvent {
    /// Opens a new stream.
    /// 
    /// Always appears before `Transmit` for the given stream `id`.
    Open {
        /// The ID for the new stream.
        id: SendStreamId,
    },

    /// Send a chunk of data over a stream.
    /// 
    /// Only occurs after an `Open` event with the same `id` is sent.
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