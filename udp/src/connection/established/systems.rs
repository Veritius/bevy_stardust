use bevy_ecs::prelude::*;
use bevy_stardust::prelude::*;
use crate::plugin::PluginConfiguration;
use crate::Connection;
use super::{frame::*, packing::*, Established};

pub(crate) fn established_packet_reader_system(
    registry: ChannelRegistry,
    config: Res<PluginConfiguration>,
    mut connections: Query<(Entity, &mut Connection, &mut Established, &mut NetworkMessages<Incoming>)>,
) {
    todo!()
}

pub(crate) fn established_packet_builder_system(
    registry: ChannelRegistry,
    config: Res<PluginConfiguration>,
    scratch: Res<PackingScratchCells>,
    mut connections: Query<(Entity, &mut Connection, &mut Established, &NetworkMessages<Outgoing>)>,
) {
    // Static context for packing manager
    let context = PackingContext {
        config: &config,
        registry: &registry,
    };

    // Iterate all peers
    connections.par_iter_mut().for_each(|(entity, mut connection, mut established, outgoing)| {
        // Span for debugging
        let trace_span = tracing::trace_span!("Building packets", peer=?entity);
        let _entered = trace_span.enter();

        // Get the packing scratch data
        let scratch_cell = scratch.cell();
        let mut scratch_data = scratch_cell.replace(PackingScratch::empty());

        // Move management frames into the packing queue
        for frame in established.frames.drain(..) {
            scratch_data.push_frame(frame);
        }

        // Move Stardust messages into the packing queue
        for (channel, messages) in outgoing.all_queues() {
            // Collect data about the messages overall
            let channel_int = u32::from(channel).wrapping_add(1);
            let channel_data = registry.channel_config(channel).unwrap();
            let mut flags = FrameFlags::default();
            if channel_data.reliable == ReliabilityGuarantee::Reliable { flags |= FrameFlags::RELIABLE; }
            if channel_data.ordered != OrderingGuarantee::Unordered { flags |= FrameFlags::ORDERED; }

            // Add all messages to queue
            for message in messages {
                scratch_data.push_frame(Frame {
                    flags,
                    ident: channel_int,
                    bytes: message.clone(),
                });
            }
        }

        // Build and run the packing manager
        let mut packing = PackingManager::build(&mut scratch_data, context);
        packing.run();

        // Return scratch data to cell
        scratch_cell.replace(scratch_data);
    });
}