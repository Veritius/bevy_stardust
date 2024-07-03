use std::time::Instant;
use bevy::prelude::*;
use bevy_stardust::prelude::*;
use bevy_stardust_extras::varint::VarInt;
use bytes::BufMut;
use smallvec::SmallVec;
use super::frames::builder::PacketBuilderContext;
use super::frames::frames::{FrameType, SendFrame};
use super::frames::header::PacketHeader;
use crate::plugin::PluginConfiguration;
use crate::prelude::*;
use crate::sequences::SequenceId;
use super::Established;

pub(in crate::connection) fn established_resend_system(
    config: Res<PluginConfiguration>,
    mut connections: Query<(&mut Connection, &mut Established)>
) {
    // Iterate all peers in parallel
    connections.par_iter_mut().for_each(|(mut connection, mut established)| {
        // Record the time we started
        let started = Instant::now();

        // Create resend filter and check if anything needs resending
        let resend = connection.congestion.get_resend();
        let filter = |time: Instant| time.duration_since(started) > resend;
        if !established.reliability.any_old(filter) { return } // Nothing to do

        // Drain the old reliability packets
        let mut resends: SmallVec<[(SequenceId, Bytes); 3]> = SmallVec::new();
        let rel_state = established.reliability.clone_state();
        let mut transient_sequence = rel_state.local_sequence.clone();
        let iter = established.reliability.drain_old(filter);

        // Serialise and resend all of these packets reliably
        let mut scr: Vec<u8> = Vec::with_capacity(connection.congestion.get_mtu());
        for packet in iter {
            // Clear the scratch space
            scr.clear();

            // Create the packet header
            let seq = transient_sequence;
            let header = PacketHeader::Reliable {
                seq,
                ack: rel_state.remote_sequence,
                bits: rel_state.ack_memory,
            };

            // Add to the sequence value
            transient_sequence += 1;

            // Serialize the packet header and add the payload.
            header.write(&mut scr, config.reliable_bitfield_length);
            scr.put(packet.payload.clone());

            // Create a new Bytes to send this frame
            let frozen = Bytes::copy_from_slice(&scr[..]);
            connection.send_queue.push_back(frozen);

            // Add to the resends set
            resends.push((seq, packet.payload.clone()));
        }

        // Put the resends back into the reliability manager
        for (sequence, payload) in resends.drain(..) {
            established.reliability.record(sequence, payload);
        }

        // Update the sequence ID in the component as we can now access it mutably
        // We couldn't previously do this as iter held a mutable borrow of established
        established.reliability.state_mut().local_sequence = transient_sequence;
    });
}

pub(in crate::connection) fn established_writing_system(
    registry: Channels,
    config: Res<PluginConfiguration>,
    mut connections: Query<(&mut Connection, &mut Established, &PeerMessages<Outgoing>)>,
) {
    // Iterate all peers in parallel
    connections.par_iter_mut().for_each(|(mut connection, mut established, messages)| {
        // Reborrows and stuff
        let established = &mut *established;
        let orderings = &mut established.orderings;
        let builder = &mut established.builder;

        // Find out how many frames we need to send
        let mut frame_total = 0;
        let outgoing_count = messages.count();
        frame_total += outgoing_count;
        frame_total += builder.unsent();

        // No frames, no work to do
        if frame_total == 0 { return; }

        // Congestion control and timing stuff
        let start = Instant::now();
        let mtu = connection.congestion.get_mtu();
        let budget = connection.congestion.get_budget(start);

        // Add all outgoing messages as frames
        if outgoing_count > 0 {
            // Iterate over all channels
            for (channel, messages) in messages.iter() {
                // Get channel data from the registry
                let config = registry.config(channel)
                    .expect(&format!("Message sent on nonexistent channel {channel:?}"));

                // Store some data about the channel so we don't repeat ourselves
                let is_ordered = config.consistency.is_ordered();
                let is_reliable = config.consistency.is_reliable();
                let channel_varint: VarInt = Into::<u32>::into(channel).into();

                // Get a new ordering if necessary
                let mut orderings = match is_ordered {
                    true => Some(orderings.get(channel, config)),
                    false => None,
                };

                // Iterate over all messages
                for message in messages {
                    // If the channel is ordered, give it an ordering sequence
                    let order = match is_ordered {
                        true => Some(orderings.as_mut().unwrap().advance()),
                        false => None,
                    };

                    // Construct the frame data
                    let frame = SendFrame {
                        priority: config.priority,
                        time: start,
                        ftype: FrameType::Stardust,
                        reliable: is_reliable,
                        order,
                        ident: Some(channel_varint),
                        payload: message.into(),
                    };

                    // Add it to the builder
                    builder.put(frame);
                }
            }
        }

        // Setup scratch space for the packet builder
        let mut scratch = Vec::with_capacity(mtu);

        let established = &mut *established;
        let reliability = &mut established.reliability;
        let builder = &mut established.builder;

        // Setup context for the builder
        let context = PacketBuilderContext {
            config: &config,
            rel_state: reliability,
            scratch: &mut scratch,
        };

        // Run the packet builder
        let mut frames = builder.run(
            budget,
            mtu,
            context
        );

        // Place the generated frames into the send queue
        let mut consumed: usize = 0;
        for frame in frames.drain(..) {
            consumed += frame.len();
            connection.send_queue.push_back(frame);
        }

        // Update congestion control
        debug_assert!(consumed <= budget);
        connection.congestion.consume_budget(consumed);
    });
}