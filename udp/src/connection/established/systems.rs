use std::cell::Cell;

use bevy_ecs::prelude::*;
use bevy_stardust::prelude::*;
use thread_local::ThreadLocal;
use crate::{connection::ordering::OrderedMessages, packet::MTU_SIZE, plugin::PluginConfiguration, Connection};
use super::Established;

macro_rules! try_unwrap {
    ($id:tt, $st:expr) => {
        match $st {
            Ok(val) => val,
            Err(_) => { continue $id; }
        }
    };
}

pub(crate) fn established_packet_reader_system(
    mut connections: Query<(Entity, &mut Connection, &mut Established, &mut NetworkMessages<Incoming>)>,
) {
    // Process all connections in parallel
    connections.par_iter_mut().for_each(|(entity, mut meta, mut state, mut incoming)| {
        // Tracing info for logging
        let span = tracing::trace_span!("Reading packets", peer=?entity);
        let _entered_span = span.enter();

        todo!()
    });
}

#[derive(Default)]
pub(crate) struct PacketBuilderSystemScratch(ThreadLocal<Cell<PacketBuilderSystemScratchInner>>);

struct PacketBuilderSystemScratchInner {
    pub msg_buffer: BytesMut,
    pub pkt_buffer: BytesMut,
    pub reliable: Vec<(ChannelId, Bytes)>,
    pub unreliable: Vec<(ChannelId, Bytes)>,
}

impl Default for PacketBuilderSystemScratchInner {
    // Default, unallocated scratch space.
    // Exists because `Cell::take` replaces the inner value with the Default implementation.
    fn default() -> Self {
        Self {
            msg_buffer: BytesMut::new(),
            pkt_buffer: BytesMut::new(),
            reliable: Vec::new(),
            unreliable: Vec::new(),
        }
    }
}

pub(crate) fn established_packet_builder_system(
    registry: ChannelRegistry,
    config: Res<PluginConfiguration>,
    scratch: Local<PacketBuilderSystemScratch>,
    mut connections: Query<(Entity, &mut Connection, &mut Established, &NetworkMessages<Outgoing>)>,
) {
    // Process all connections in parallel
    connections.par_iter_mut().for_each(|(entity, mut meta, mut state, outgoing)| {
        // Tracing info for logging
        let span = tracing::trace_span!("Building packets", peer=?entity);
        let _entered_span = span.enter();

        // Fetch or create the thread local scratch space
        let scratch_cell = scratch.0.get_or(|| Cell::new(PacketBuilderSystemScratchInner {
            // These seem like reasonable defaults.
            msg_buffer: BytesMut::with_capacity(MTU_SIZE),
            pkt_buffer: BytesMut::with_capacity(MTU_SIZE),
            reliable: Vec::with_capacity(32),
            unreliable: Vec::with_capacity(256),
        }));

        // Take out the inner scratch
        let mut scratch = scratch_cell.take();

        // Sort messages into queues
        let mut queues = outgoing.all_queues();
        while let Some((channel, payloads)) = queues.next() {
            // Get channel data
            let channel_data = registry.channel_config(channel).unwrap();
            let is_reliable = channel_data.reliable == ReliabilityGuarantee::Reliable;

            // Iterate over all payloads
            for payload in payloads {
                match is_reliable {
                    true => scratch.reliable.push((channel, payload.clone())),
                    false => scratch.reliable.push((channel, payload.clone())),
                }
            }
        }

        // Sort the messages in queues by their size
        [&mut scratch.reliable, &mut scratch.unreliable].into_iter()
            .for_each(|v| v.sort_unstable_by(|(_,a),(_,b)| {
                a.len().cmp(&b.len())
            }));

        // Record how many messages we have queued
        if !span.is_disabled() {
            span.record("reliable messages", scratch.reliable.len());
            span.record("unreliable messages", scratch.unreliable.len());
        }

        todo!();

        // Return scratch to the cell
        scratch.msg_buffer.clear();
        scratch.pkt_buffer.clear();
        scratch.reliable.clear();
        scratch.unreliable.clear();
        scratch_cell.set(scratch);
    });
}