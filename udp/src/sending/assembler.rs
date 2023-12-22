//! Assembling octet strings into packets.

use std::ops::IndexMut;
use bevy_stardust::prelude::*;
use crate::{prelude::*, utils::bytes_for_channel_ids, reliability::pipe_for_channel, MAXIMUM_TRANSPORT_UNITS};
use super::packing::best_fit;

pub(super) fn assemble_packets<'a>(
    config: &UdpPluginConfig,
    channels: &ChannelRegistry,
    peer_data: &mut UdpConnection,
    strings: impl Iterator<Item = (ChannelId, &'a Bytes)>,
) -> Box<[Box<[u8]>]> {
    let channel_count = channels.channel_count();
    let channel_id_bytes = bytes_for_channel_ids(channel_count);

    // Bins for packing octet strings
    let mut unreliable_bins: Vec<Packet> = Vec::with_capacity(1);
    let mut reliable_pipe_bins: Vec<Vec<Packet<(u16, u16, u32)>>> = Vec::with_capacity(config.reliable_pipes as usize);
    reliable_pipe_bins.fill_with(|| Vec::default());

    // Scratch space for working
    let mut scratch = [0u8; 1450];

    // Iterate over all strings and pack them
    for (channel, string) in strings {
        let mut length: usize = 0;

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
        write_scratch(&mut scratch, &mut length, &string);

        // Reliable messages have a different process
        let buffer = &scratch[..length];
        match channel_data.reliable {
            false => pack_bytes(
                &mut unreliable_bins,
                buffer,
            ),
            true => {
                // Pipe count check
                if config.reliable_pipes == 0 {
                    #[cfg(not(debug_assertions))] {
                        pack_bytes(&mut unreliable_bins, buffer);
                        error!("A reliable message was queued for sending, but the amount of reliable pipes is zero. The message has been sent unreliably and may be lost.");
                        continue
                    }

                    #[cfg(debug_assertions)]
                    panic!("A reliable message was queued for sending, but the amount of reliable pipes is zero.");
                }
                
                let pipe_bins = reliable_pipe_bins.index_mut(
                    pipe_for_channel(config.reliable_pipes, channel_count, channel.into()) as usize);

                pack_bytes(
                    pipe_bins,
                    buffer,
                )
            },
        }
    }

    todo!()
}

pub(super) struct Packet<H: Default = ()> {
    pub header: H,
    pub data: Vec<u8>,
}

fn pack_bytes<H: Default>(
    bins: &mut Vec<Packet<H>>,
    buffer: &[u8],
) {
    let header_size = std::mem::size_of::<H>();
    let bin = best_fit(bins.iter().map(|f| (f.data.capacity() - header_size, f.data.len())), buffer.len());
    let bin = match bin {
        usize::MAX => {
            bins.push(Packet {
                header: H::default(),
                data: Vec::with_capacity(MAXIMUM_TRANSPORT_UNITS - header_size),
            });
            bins.last_mut().unwrap()
        },
        _ => {
            bins.index_mut(bin)
        },
    };

    for v in buffer {
        bin.data.push(*v);
    }
}