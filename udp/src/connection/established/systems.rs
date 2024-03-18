use std::cell::Cell;

use bevy_ecs::prelude::*;
use bevy_stardust::prelude::*;
use thread_local::ThreadLocal;
use crate::{packet::MTU_SIZE, plugin::PluginConfiguration, Connection};
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
    mut connections: Query<(&mut Connection, &mut Established, &mut NetworkMessages<Incoming>)>,
) {
    // Process all connections in parallel
    connections.par_iter_mut().for_each(|(mut meta, mut state, mut incoming)| {
        todo!()
    });
}

#[derive(Default)]
pub(crate) struct PacketBuilderSystemScratch(ThreadLocal<Cell<PacketBuilderSystemScratchInner>>);

struct PacketBuilderSystemScratchInner {
    pub bytes: Vec<u8>,
    pub reliable: Vec<(ChannelId, Bytes)>,
    pub unreliable: Vec<(ChannelId, Bytes)>,
}

impl Default for PacketBuilderSystemScratchInner {
    // Default, unallocated scratch space.
    // Exists because `Cell::take` replaces the inner value with the Default implementation.
    fn default() -> Self {
        Self {
            bytes: Vec::with_capacity(0),
            reliable: Vec::with_capacity(0),
            unreliable: Vec::with_capacity(0),
        }
    }
}

pub(crate) fn established_packet_builder_system(
    registry: ChannelRegistry,
    config: Res<PluginConfiguration>,
    scratch: Local<PacketBuilderSystemScratch>,
    mut connections: Query<(&mut Connection, &mut Established, &NetworkMessages<Outgoing>)>,
) {
    // Process all connections in parallel
    connections.par_iter_mut().for_each(|(mut meta, mut state, outgoing)| {
        // Fetch or create the thread local scratch space
        let scratch_cell = scratch.0.get_or(|| Cell::new(PacketBuilderSystemScratchInner {
            // These seem like reasonable defaults.
            bytes: Vec::with_capacity(MTU_SIZE),
            reliable: Vec::with_capacity(32),
            unreliable: Vec::with_capacity(256),
        }));

        // Take out the inner scratch
        let mut scratch = scratch_cell.take();

        // Iterate over all queues to collect messages
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

        // Return scratch
        scratch_cell.set(scratch);
    });
}