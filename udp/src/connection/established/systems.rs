use std::{cell::Cell, cmp::Ordering, ops::{BitOr, BitOrAssign}};
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
    pub byte_bins: Vec<BytesMut>,
    pub messages: Vec<Message>,
}

impl Default for PacketBuilderSystemScratchInner {
    // Default, unallocated scratch space.
    // Exists because `Cell::take` replaces the inner value with the Default implementation.
    fn default() -> Self {
        Self {
            msg_buffer: BytesMut::new(),
            byte_bins: Vec::new(),
            messages: Vec::new(),
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
            byte_bins: Vec::with_capacity(16),
            messages: Vec::with_capacity(256),
        }));

        // Take out the inner scratch
        let mut scratch = scratch_cell.take();

        // Push messages to scratch queue with extra data
        let mut queues = outgoing.all_queues();
        while let Some((channel, payloads)) = queues.next() {
            // Get channel data
            let channel_data = registry.channel_config(channel).unwrap();
            let is_reliable = channel_data.reliable == ReliabilityGuarantee::Reliable;
            let is_ordered= channel_data.ordered != OrderingGuarantee::Unordered;

            // Flags for channel type
            let mut flags = MessageFlags::DEFAULT;
            if is_reliable { flags |= MessageFlags::RELIABLE; }
            if is_ordered { flags |= MessageFlags::ORDERED; }

            // Iterate over all payloads
            for payload in payloads {
                scratch.messages.push(Message {
                    channel,
                    payload: payload.clone(),
                    flags,
                });
            }
        }

        // Sort messages by reliable flag and size
        scratch.messages.sort_unstable_by(|a, b| {
            // Check the reliable flags
            match (a.flags.is_reliable(), b.flags.is_reliable()) {
                (true, false) => { return Ordering::Greater },
                (false, true) => { return Ordering::Less },
                _ => {}
            }

            // Return payload length
            return a.payload.len().cmp(&b.payload.len());
        });

        // Process all messages
        for message in scratch.messages.iter() {
            // Put the channel id into the buffer
            scratch.msg_buffer.put_u32(u32::from(message.channel).wrapping_add(1));

            // If present, put ordering data into buffer
            if message.flags.is_ordered() {
                let ordering_data = state.ordering(message.channel);
                scratch.msg_buffer.put_u16(ordering_data.advance());
            }

            // Put the message payload into the buffer
            scratch.msg_buffer.put(&*message.payload);

            todo!()
        }

        // Clean up after ourselves and return scratch to the cell
        scratch.msg_buffer.clear();
        scratch.byte_bins.clear();
        scratch.messages.clear();
        scratch_cell.set(scratch);
    });
}

struct Message {
    channel: ChannelId,
    payload: Bytes,
    flags: MessageFlags,
}

#[derive(Clone, Copy)]
#[repr(transparent)]
struct MessageFlags(u32);

impl MessageFlags {
    const DEFAULT: Self = Self(0);

    const RELIABLE: Self = Self(1 << 0);
    const ORDERED: Self = Self(1 << 1);

    #[inline]
    fn is_reliable(&self) -> bool {
        (self.0 & !Self::RELIABLE.0) > 0
    }

    #[inline]
    fn is_ordered(&self) -> bool {
        (self.0 & !Self::ORDERED.0) > 0
    }
}

impl BitOr for MessageFlags {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for MessageFlags {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        *self = self.bitor(rhs)
    }
}