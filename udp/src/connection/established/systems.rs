use bevy_ecs::prelude::*;
use bevy_stardust::prelude::*;
use unbytes::*;
use crate::{packet::OutgoingPacket, plugin::PluginConfiguration};
use crate::Connection;
use super::packet::Frame;
use super::parsing::{PacketHeaderData, ParsedFrame, FrameParseError};
use super::{packing::*, Established};

pub(crate) fn established_packet_reader_system(
    registry: ChannelRegistry,
    config: Res<PluginConfiguration>,
    mut connections: Query<(Entity, &mut Connection, &mut Established, &mut NetworkMessages<Incoming>)>,
) {
    // Iterate all peers
    connections.par_iter_mut().for_each(|(entity, mut connection, mut established, mut incoming)| {
        // Skip connections without any incoming packets
        if connection.packet_queue.incoming().len() == 0 { return; }

        // Span for debugging
        let trace_span = tracing::debug_span!("Reading packets", peer=?entity);
        let _entered = trace_span.enter();

        // Pop incoming packets
        'packet: while let Some(packet) = connection.packet_queue.pop_incoming() {
            // Span for each packet
            let trace_span = tracing::trace_span!("Reading packet");
            let _entered = trace_span.enter();

            // Reader to process the packet
            let mut reader = Reader::new(packet.payload);

            // Get the packet header
            let header = match PacketHeaderData::parse(&mut reader, &config) {
                Ok(v) => v,
                Err(_) => {
                    // TODO: Handle this case.
                    continue 'packet;
                },
            };

            // If the packet is reliable, acknowledge it
            if let Some(reliable) = header.reliable {
                established.reliability.ack(reliable, config.reliable_bitfield_length as u8);
            }

            // Repeatedly parse frames
            'frame: loop {
                // Parse a frame header
                let parsed = match ParsedFrame::parse(&mut reader, &config, &registry) {
                    Ok(v) => v,
                    Err(FrameParseError::EndOfInput) => { break 'packet; },
                };

                // Get the message data
                let message = match reader.read_bytes(parsed.length) {
                    Ok(v) => v,
                    Err(_) => { break 'packet; },
                };

                // Forward it to the correct whatever
                if parsed.ident > 0 {
                    // This is a message frame
                    let ident = parsed.ident.wrapping_sub(1);
                    incoming.push(ChannelId::from(ident), message);
                } else {
                    // This is a transport frame
                    todo!()
                }
            }
        }
    });
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
        let trace_span = tracing::debug_span!("Building packets", peer=?entity);
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

            let mut flags = 0u32;
            if channel_data.reliable == ReliabilityGuarantee::Reliable { flags |= Frame::IS_RELIABLE; }
            if channel_data.ordered != OrderingGuarantee::Unordered { flags |= Frame::IS_ORDERED; }

            // Add all messages to queue
            for message in messages {
                scratch_data.push_frame(Frame {
                    flags,
                    ident: channel_int,
                    bytes: message.clone(),
                });
            }
        }

        // Build and run the packing instance
        let mut instance = PackingInstance::build(&mut established, &mut scratch_data, context);
        let mut finished = instance.run();

        // Queue all finished packets for sending
        while let Some(finished) = finished.next() {
            connection.packet_queue.push_outgoing(OutgoingPacket {
                payload: finished.full(),
                messages: 0, // TODO
            });
        }

        // Manually drop finished to please
        // our lord the borrow checker.
        drop(finished);

        // Return scratch data to cell
        scratch_cell.replace(scratch_data);
    });
}