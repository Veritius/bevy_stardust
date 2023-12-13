//! Assembling octet strings into packets.

use std::ops::IndexMut;
use bevy::prelude::*;
use bevy_stardust::prelude::*;
use crate::{prelude::*, utils::bytes_for_channel_ids, reliability::{ReliabilityData, pipe_for_channel}, MAXIMUM_TRANSPORT_UNITS};

use super::packing::best_fit;

pub(super) fn assemble_packets<'a>(
    config: &UdpPluginConfig,
    channels: &ChannelRegistry,
    peer_data: &mut UdpConnection,
    strings: impl Iterator<Item = (ChannelId, &'a OctetString)>,
) -> Box<[Box<[u8]>]> {
    let channel_count = channels.channel_count();
    let channel_id_bytes = bytes_for_channel_ids(channel_count);

    // Bins for packing octet strings
    let mut unreliable_bins: Vec<Vec<u8>> = Vec::with_capacity(1);
    let mut reliable_bins: Vec<Vec<ReliablePacket>> = Vec::with_capacity(config.reliable_pipes as usize);
    reliable_bins.fill_with(|| Vec::default());

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
        write_scratch(&mut scratch, &mut length, string.as_slice());

        // Reliable messages have a different process
        match channel_data.reliable {
            false => unreliable(
                &mut unreliable_bins,
                &scratch[..length],
            ),
            true => {
                // Pipe count check
                if config.reliable_pipes == 0 {
                    #[cfg(not(debug_assertions))] {
                        unreliable(
                            &mut unreliable_bins,
                            &scratch[..length],
                        );
                        error!("A reliable message was queued for sending, but the amount of reliable pipes is zero. The message has been sent unreliably and may be lost.");
                        continue
                    }

                    #[cfg(debug_assertions)]
                    panic!("A reliable message was queued for sending, but the amount of reliable pipes is zero.");
                }
                
                let bin = &mut reliable_bins.index_mut(
                    pipe_for_channel(config.reliable_pipes, channel_count, channel.into()) as usize);

                reliable(
                    bin,
                    &scratch[..length],
                    &mut peer_data.reliability,
                )
            },
        }
    }

    todo!()
}

fn unreliable(
    bins: &mut Vec<Vec<u8>>,
    buffer: &[u8],
) {
    let bin = best_fit(bins, buffer.len());
    let bin = match bin {
        usize::MAX => {
            let mut vec = Vec::with_capacity(MAXIMUM_TRANSPORT_UNITS);
            vec.push(0); // unreliable 'pipe' has id zero
            bins.push(vec);
            bins.last_mut().unwrap()
        },
        _ => {
            bins.index_mut(bin)
        },
    };

    for v in buffer {
        bin.push(*v);
    }
}

pub(super) struct ReliablePacket {
    pub seq: u16,
    pub ack: u16,
    pub ack_bits: u32,
    pub data: Vec<u8>,
}

fn reliable(
    bins: &mut Vec<ReliablePacket>,
    buffer: &[u8],
    peer_data: &mut ReliabilityData,
) {
    todo!()
}