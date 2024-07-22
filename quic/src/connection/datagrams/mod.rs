mod channels;
mod header;
mod incoming;
mod outgoing;

pub(crate) use channels::ChannelDatagrams;
pub(crate) use incoming::IncomingDatagrams;
pub(crate) use outgoing::OutgoingDatagrams;

use crate::backend::DatagramManager;

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