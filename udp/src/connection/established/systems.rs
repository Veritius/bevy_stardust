use bevy_ecs::prelude::*;
use bevy_stardust::prelude::*;
use crate::{packet::MTU_SIZE, plugin::PluginConfiguration, Connection};
use super::{packing::BestFit, Established};

macro_rules! try_unwrap {
    ($id:tt, $st:expr) => {
        match $st {
            Ok(val) => val,
            Err(_) => { continue $id; }
        }
    };
}

pub(crate) fn established_packet_reader_system(
    mut connections: Query<(&mut Connection, &mut Established, &mut NetworkMessages<Incoming>)>,
) {
    // Process all connections in parallel
    connections.par_iter_mut().for_each(|(mut meta, mut state, mut incoming)| {
        todo!()
    });
}

pub(crate) fn established_packet_builder_system(
    registry: ChannelRegistry,
    config: Res<PluginConfiguration>,
    mut connections: Query<(&mut Connection, &mut Established, &NetworkMessages<Outgoing>)>,
) {
    // Process all connections in parallel
    connections.par_iter_mut().for_each(|(mut meta, mut state, outgoing)| {
        // Scratch space
        let mut scratch = BytesMut::with_capacity(MTU_SIZE);

        // Include an alt message for every N main messages
        let msg_queue_len = outgoing.count();
        let alt_queue_len = state.frames.len();
        let msg_alt_nfrac = msg_queue_len / alt_queue_len;

        // Iterator for individual messages and their channel ids
        let mut msg_queue = outgoing
            .all_queues()
            .flat_map(|(c,s)| s.iter().map(move |v| (c,v)));

        // Iterator for queued alt-messages
        let mut alt_queue_swap = Vec::new();
        std::mem::swap(&mut alt_queue_swap, &mut state.frames);
        let mut alt_queue = alt_queue_swap.drain(..);

        // Iterates over all messages
        let mut msg_idx = 0;
        let mut tf_count = 0;
        while msg_idx < msg_queue_len {
            if tf_count < msg_alt_nfrac {
                // Get the message
                let (channel, message) = msg_queue.next().unwrap();

                // Put channel id, shifted by 1 to make space for reserved values
                let channel_int = u32::from(channel).checked_add(1).unwrap();
                scratch.put_u32(channel_int);

                // Retrieve channel data
                let channel_data = registry.channel_config(channel).unwrap();
                let is_ordered = channel_data.ordered != OrderingGuarantee::Unordered;
                let is_reliable = channel_data.reliable == ReliabilityGuarantee::Reliable;

                // Ordering header
                if is_ordered {
                    let ordered_messages = state.ordering(channel);
                    let index = ordered_messages.advance();
                    scratch.put_u16(index);
                }

                // Push the payload (SOMEBODY STOP THAT CART!)
                scratch.put(&**message);

                // Put into packer
                match is_reliable {
                    true => state.reliable_packer.push::<BestFit>(&scratch),
                    false => state.unreliable_packer.push::<BestFit>(&scratch),
                }

                // Update trackers for next iteration
                msg_idx += 1;
                tf_count += 1;
                scratch.clear();
                continue
            } else {
                // Get the message
                let message = alt_queue.next().unwrap();

                // Reserved value for alt messages
                scratch.put_u32(0);

                // Alt frame type
                scratch.put_u8(message.id as u8);

                // Push the payload
                scratch.put(&*message.pld);

                // Put into packer
                state.reliable_packer.push::<BestFit>(&scratch);

                // Update trackers for next iteration
                tf_count = 0;
                scratch.clear();
                continue
            }
        }

        // Return the swapped vec to the table
        drop(alt_queue); // Manual drop to appease borrowck
        std::mem::swap(&mut alt_queue_swap, &mut state.frames);

        // Clear scratch
        // This probably isn't necessary but we do it just in case
        scratch.clear();

        // Pop all bins from the reliable packer
        while let Some(bin) = state.reliable_packer.pop(|_| true) {
            todo!();

            // Clean up for the next iteration
            scratch.clear();
        }

        while let Some(bin) = state.unreliable_packer.pop(|_| true) {
            todo!();

            scratch.clear();
            // Clean up for the next iteration
        }
    });
}