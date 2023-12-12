//! Assembling octet strings into packets.

use bevy_stardust::prelude::*;
use crate::{prelude::*, utils::bytes_for_channel_ids};

pub(super) fn assemble_packets<'a>(
    channels: &ChannelRegistry,
    peer_data: &mut UdpConnection,
    strings: impl Iterator<Item = (ChannelId, &'a OctetString)>,
) -> Box<[Box<[u8]>]> {
    let channel_id_bytes = bytes_for_channel_ids(channels.channel_count());

    // Bins for packing octet strings
    let mut unreliable_bins: Vec<Vec<u8>> = Vec::with_capacity(1);
    let mut reliable_bins: Vec<Vec<u8>> = Vec::with_capacity(1);

    // Iterate over all strings and pack them
    for (channel, string) in strings {
        let channel_data = channels.get_from_id(channel).unwrap();

        // Reliable messages have a different process
        match channel_data.reliable {
            false => unreliable(
                
            ),
            true => reliable(

            ),
        }
    }

    todo!()
}

fn unreliable(

) {
    todo!()
}

fn reliable(

) {
    todo!()
}