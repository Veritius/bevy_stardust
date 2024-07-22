mod channels;
mod header;
mod incoming;
mod outgoing;

pub(super) use channels::ChannelStreams;
pub(super) use incoming::IncomingStreams;
pub(super) use outgoing::{OutgoingStreams, OutgoingStreamsTryWriteOutcome};

use bevy_stardust::prelude::*;
use crate::backend::StreamId;

#[derive(Debug, Clone, Copy)]
pub(super) enum StreamTag {
    Stardust { channel: ChannelId },
    Datagram,
}