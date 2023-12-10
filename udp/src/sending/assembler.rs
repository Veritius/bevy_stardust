//! Assembling octet strings into packets.

use bevy_stardust::prelude::*;
use crate::prelude::*;

pub(super) fn assemble_packets<'a>(
    channels: &ChannelRegistry,
    peer_data: &mut UdpConnection,
    strings: impl Iterator<Item = (ChannelId, &'a OctetString)>,
) -> Box<[Box<[u8]>]> {
    for (channel, string) in strings {
        let channel_data = channels.get_from_id(channel).unwrap();
    }

    todo!()
}