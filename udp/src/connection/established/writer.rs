use std::time::Instant;
use bevy::prelude::*;
use bevy_stardust::prelude::*;
use crate::connection::packets::builder::PacketBuilderContext;
use crate::connection::packets::frames::{FrameFlags, FrameType, SendFrame};
use crate::plugin::PluginConfiguration;
use crate::prelude::*;
use crate::varint::VarInt;
use super::Established;

pub(crate) fn established_writing_system(
    registry: Res<ChannelRegistry>,
    config: Res<PluginConfiguration>,
    mut connections: Query<(Entity, &mut Connection, &mut Established, &NetworkMessages<Outgoing>)>,
) {
    // Iterate all peers in parallel
    connections.par_iter_mut().for_each(|(entity, mut connection, mut established, messages)| {
        // Reborrows and stuff
        let established = &mut *established;
        let controller = &mut established.controller;
        let orderings = &mut established.orderings;
        let builder = &mut established.builder;

        // Let the controller push any new frames
        controller.send_control_frame(builder);

        // Find out how many frames we need to send
        let mut frame_total = 0;
        let outgoing_count = messages.count();
        frame_total += outgoing_count;
        frame_total += builder.unsent();

        // No frames, no work to do
        if frame_total == 0 { return; }

        // Some timing information
        let con_proc_start = Instant::now();

        {
            // Update the connection budget
            // This is in its own scope so we don't clutter things with variables
            let secs_since_last = connection.budget_ltime.duration_since(con_proc_start).as_secs_f64();
            let change_in_bytes = (secs_since_last * connection.budget_limit as f64) as usize;
            connection.budget_count += change_in_bytes;
            connection.budget_ltime = con_proc_start;
        }

        // Add all outgoing messages as frames
        if outgoing_count > 0 {
            // Iterate over all channels
            for (channel, messages) in messages.iter() {
                // Get channel data from the registry
                let channel_data = registry.channel_config(channel)
                    .expect(&format!("Message sent on nonexistent channel {channel:?}"));

                // Store some data about the channel so we don't repeat ourselves
                let is_ordered = channel_data.ordered != OrderingGuarantee::Unordered;
                let is_reliable = channel_data.reliable == ReliabilityGuarantee::Reliable;
                let channel_varint: VarInt = Into::<u32>::into(channel).into();

                // Create the flags for all frames
                // This is fine since all messages are similar
                let mut flags = FrameFlags::IDENTIFIED;
                if is_ordered {
                    flags |= FrameFlags::ORDERED;
                }

                // Get a new ordering if necessary
                let mut orderings = match is_ordered {
                    true => Some(orderings.get(channel_data)),
                    false => None,
                };

                // Iterate over all messages
                for message in messages.iter().cloned() {
                    // If the channel is ordered, give it an ordering sequence
                    let order = match is_ordered {
                        true => Some(orderings.as_mut().unwrap().advance()),
                        false => None,
                    };

                    // Construct the frame data
                    let frame = SendFrame {
                        priority: channel_data.priority,
                        time: con_proc_start,
                        flags,
                        ftype: FrameType::Stardust,
                        reliable: is_reliable,
                        order,
                        ident: Some(channel_varint),
                        payload: message,
                    };

                    // Add it to the builder
                    builder.put(frame);
                }
            }
        }

        // Setup scratch space for the packet builder
        let mut scratch = Vec::with_capacity(connection.mtu_limit);

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
            connection.budget_count,
            connection.mtu_limit,
            context
        );

        // Place the generated frames into the send queue
        for frame in frames.drain(..) {
            connection.send_queue.push_back(frame);
        }
    });
}