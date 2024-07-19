mod channels;
mod header;
mod incoming;
mod outgoing;
mod traits;

pub(crate) use channels::ChannelDatagrams;
pub(crate) use incoming::IncomingDatagrams;
pub(crate) use outgoing::OutgoingDatagrams;
pub(crate) use traits::*;

use bytes::Bytes;
use bevy_stardust::prelude::*;
use bevy_stardust_extras::numbers::Sequence;

#[derive(Debug, Clone)]
pub(crate) struct Datagram {
    pub tag: DatagramTag,
    pub payload: Bytes,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum DatagramTag {
    Stardust {
        channel: ChannelId,
        sequence: Option<Sequence<u16>>,
    },
}