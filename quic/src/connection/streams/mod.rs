mod channels;
mod header;
mod id;
mod incoming;
mod outgoing;
mod traits;

pub(super) use channels::ChannelStreams;
pub(super) use incoming::IncomingStreams;
pub(super) use outgoing::{OutgoingStreams, OutgoingStreamsTryWriteOutcome};

pub use id::StreamId;
pub use traits::*;

use bevy_stardust::prelude::*;

#[derive(Debug, Clone, Copy)]
pub(super) enum StreamTag {
    Stardust { channel: ChannelId },
    Datagram,
}