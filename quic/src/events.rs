use bevy_stardust::prelude::ChannelMessage;
use bytes::Bytes;
use crate::{RecvStreamId, SendStreamId};

/// An event sent by the state machine to the application.
pub enum ConnectionEvent {
    /// A Stardust network message was received.
    Message(ChannelMessage),

    /// A stream event occurred.
    Stream(StreamEvent),

    /// The connection is closing but not yet drained.
    Closing,

    /// The connection is drained and can be dropped.
    Drained,
}

pub enum StreamEvent {
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