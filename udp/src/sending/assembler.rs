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

    // Scratch space for working
    let mut scratch = [0u8; 1450];
    let mut length: usize = 0;

    // Iterate over all strings and pack them
    for (channel, string) in strings {
        // Reset working space
        length = 0;

        #[inline]
        fn write_scratch(
            scratch: &mut [u8],
            length: &mut usize,
            data: &[u8],
        ) {
            scratch[*length..*length+data.len()].clone_from_slice(data);
            *length += data.len();
        }

        // Write the channel ID
        let bytes = Into::<[u8;4]>::into(channel);
        match channel_id_bytes {
            // TODO: Reduce repetition here
            1 => write_scratch(&mut scratch, &mut length, &[bytes[3]]),
            2 => write_scratch(&mut scratch, &mut length, &[bytes[2], bytes[3]]),
            3 => write_scratch(&mut scratch, &mut length, &[bytes[1], bytes[2], bytes[3]]),
            4 => write_scratch(&mut scratch, &mut length, &bytes),
            _ => panic!(), // shouldn't happen
        }

        // Channel data
        let channel_data = channels.get_from_id(channel).unwrap();

        // Ordering data
        if channel_data.ordered {
            todo!()
        }

        // Write the length of the message
        write_scratch(&mut scratch, &mut length, &Into::<u16>::into(string.len() as u16).to_be_bytes());

        // Write the contents of the message
        write_scratch(&mut scratch, &mut length, string.as_slice());

        // Reliable messages have a different process
        match channel_data.reliable {
            false => unreliable(
                peer_data,
                &scratch[..length],
            ),
            true => reliable(
                peer_data,
                &scratch[..length],
            ),
        }
    }

    todo!()
}

fn unreliable(
    peer_data: &mut UdpConnection,
    buffer: &[u8],
) {
    todo!()
}

fn reliable(
    peer_data: &mut UdpConnection,
    buffer: &[u8],
) {
    todo!()
}