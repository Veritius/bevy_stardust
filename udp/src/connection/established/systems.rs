use std::{cell::Cell, cmp::Ordering, ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign}};
use bevy_ecs::prelude::*;
use bevy_stardust::prelude::*;
use thread_local::ThreadLocal;
use crate::{packet::{OutgoingPacket, MTU_SIZE}, plugin::PluginConfiguration, Connection};
use super::{frame::PacketHeader, Established};

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
    pub bytes: BytesMut,
    pub messages: Vec<Message>,
    pub bins: Vec<Bin>,
}

impl Default for PacketBuilderSystemScratchInner {
    // Default, unallocated scratch space.
    // Exists because `Cell::take` replaces the inner value with the Default implementation.
    fn default() -> Self {
        Self {
            bytes: BytesMut::new(),
            messages: Vec::new(),
            bins: Vec::new(),
        }
    }
}

pub(crate) fn established_packet_builder_system(
    registry: ChannelRegistry,
    config: Res<PluginConfiguration>,
    scratch: Local<PacketBuilderSystemScratch>,
    mut connections: Query<(Entity, &mut Connection, &mut Established, &NetworkMessages<Outgoing>)>,
) {
    const BLANK_PREFIX_LENGTH: usize = 32;
    const PKT_LEN_WITH_PREFIX: usize = MTU_SIZE + BLANK_PREFIX_LENGTH;

    // Process all connections in parallel
    connections.par_iter_mut().for_each(|(entity, mut meta, mut state, outgoing)| {
        // Tracing info for logging
        let span = tracing::trace_span!("Building packets", peer=?entity);
        let _entered_span = span.enter();

        // Fetch or create the thread local scratch space
        let scratch_cell = scratch.0.get_or(|| Cell::new(PacketBuilderSystemScratchInner {
            // These seem like reasonable defaults.
            bytes: BytesMut::with_capacity(MTU_SIZE),
            bins: Vec::with_capacity(16),
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
            scratch.bytes.put_u32(u32::from(message.channel).wrapping_add(1));

            // If present, put ordering data into buffer
            if message.flags.is_ordered() {
                let ordering_data = state.ordering(message.channel);
                scratch.bytes.put_u16(ordering_data.advance());
            }

            // Put the message payload into the buffer
            scratch.bytes.put(&*message.payload);

            // Use the first-fit algorithm to find a bin
            let bin = {
                // Check if a bin with sufficient remaining space exists
                let bin = scratch.bins.iter_mut()
                .find(|v| {
                    let capacity = v.buffer.capacity() - BLANK_PREFIX_LENGTH;
                    let remaining = capacity - v.buffer.len();
                    remaining >= scratch.bytes.len()
                });

                // Return it if we found something, create it if not
                match bin {
                    Some(v) => v,
                    None => {
                        // Construct header
                        let mut header = PacketHeader::new();
                        let is_reliable = message.flags.is_reliable();
                        if is_reliable { header |= PacketHeader::FLAG_RELIABLE; }

                        // Construct buffer
                        let mut buffer = Vec::with_capacity(PKT_LEN_WITH_PREFIX);
                        buffer.extend_from_slice(&[0u8; BLANK_PREFIX_LENGTH]);
                        scratch.bins.push(Bin { header, buffer, messages: 0 });
                        scratch.bins.last_mut().unwrap()
                    },
                }
            };

            // Extend the bin by the message buffer, and clear the message buffer
            bin.buffer.extend_from_slice(&scratch.bytes);
            bin.messages += 1;
            scratch.bytes.clear();
        }

        // Add bins to the send queue after some tweaks
        for mut bin in scratch.bins.drain(..) {
            // Some variables about the bin
            let is_reliable = bin.header.is_reliable();
            let mut sequence = 0; // not relevant until later

            // Append the packet header
            scratch.bytes.put_u16(bin.header.into());

            // If the packet is reliable, append a packet header.
            if is_reliable {
                // Create header
                let header = state.reliability.header();
                state.reliability.increment_local();
                sequence = header.sequence;

                // Write header integers
                scratch.bytes.put_u16(header.sequence);
                scratch.bytes.put_u16(header.ack);

                // Write the bitfield
                let bitfield_bytes = header.ack_bitfield.to_be_bytes();
                scratch.bytes.put(&bitfield_bytes[..config.reliable_bitfield_length as usize]);
            }

            // Check if the header scratch has overrun the available space
            // If it has, the next operations will corrupt the packet payloads
            assert!(scratch.bytes.len() <= BLANK_PREFIX_LENGTH);

            // Write the buffer to the packet allocation
            let length = scratch.bytes.len();
            let offset = BLANK_PREFIX_LENGTH - length;
            bin.buffer[offset..BLANK_PREFIX_LENGTH].copy_from_slice(&scratch.bytes);

            // To avoid reallocation, fill the rest of the bin with zeros
            // This is because Bytes' impl From<Vec<u8>> will reallocate to fit the length
            // While this wastes a bit of memory, it's freed in the next system, so it's fine.
            let total_len = bin.buffer.len();
            bin.buffer.extend((0..(bin.buffer.capacity() - total_len)).map(|_| 0));
            debug_assert_eq!(bin.buffer.len(), bin.buffer.capacity());

            // Turn into a Bytes object and slice it up a bit
            // This is required because we have a fair bit of bytes in the buffer
            // that would be useless at best (and harmful at most) to send
            let full = Bytes::from(bin.buffer).slice(offset..total_len);
            let payload = full.slice(..length);

            // Reliable packets need to be stored until acked
            if is_reliable {
                state.reliability.record(sequence, payload.clone());
            }

            // Finally, put it in the buffer for sending
            meta.packet_queue.push_outgoing(OutgoingPacket {
                payload,
                messages: bin.messages,
            });

            // Clear the buffer for the next iteration
            scratch.bytes.clear();
        }

        // Clean up after ourselves and return scratch to the cell
        scratch.bytes.clear();
        scratch.messages.clear();
        scratch.bins.clear();
        scratch_cell.set(scratch);
    });
}

struct Bin {
    header: PacketHeader,
    messages: u32,
    buffer: Vec<u8>,
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
        (*self & Self::RELIABLE).0 > 0
    }

    #[inline]
    fn is_ordered(&self) -> bool {
        (*self & Self::ORDERED).0 > 0
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

impl BitAnd for MessageFlags {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for MessageFlags {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        *self = self.bitand(rhs);
    }
}