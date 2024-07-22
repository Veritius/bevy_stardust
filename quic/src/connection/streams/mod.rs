mod channels;
mod header;
mod id;
mod incoming;
mod outgoing;
mod traits;

pub(crate) use channels::ChannelStreams;
pub(crate) use id::StreamId;
pub(crate) use incoming::IncomingStreams;
pub(crate) use outgoing::{OutgoingStreams, OutgoingStreamsTryWriteOutcome};
pub(crate) use traits::*;

use bevy_stardust::prelude::*;

#[derive(Debug, Clone, Copy)]
pub(super) enum StreamTag {
    Stardust { channel: ChannelId },
    Datagram,
}