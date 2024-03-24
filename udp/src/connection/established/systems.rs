use std::{cell::Cell, cmp::Ordering, ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign}, time::Instant};
use bevy_ecs::prelude::*;
use bevy_stardust::prelude::*;
use thread_local::ThreadLocal;
use unbytes::Reader;
use crate::{connection::{ordering::{OrderedMessage, OrderedMessages}, reliability::ReliablePacketHeader}, packet::{OutgoingPacket, MTU_SIZE}, plugin::PluginConfiguration, sequences::SequenceId, Connection};
use super::{frame::PacketHeader, Established};

macro_rules! try_read {
    ($op:expr, $fail:expr) => {
        match $op {
            Ok(v) => v,
            Err(_) => { $fail; }
        }
    };
}

pub(crate) fn established_packet_reader_system(
    registry: ChannelRegistry,
    config: Res<PluginConfiguration>,
    mut connections: Query<(Entity, &mut Connection, &mut Established, &mut NetworkMessages<Incoming>)>,
) {
    // Process all connections in parallel
    connections.par_iter_mut().for_each(|(entity, mut meta, mut state, mut incoming)| {
        // Check if there's anything to process
        if meta.packet_queue.incoming().len() == 0 { return }

        // Tracing info for logging
        let span = tracing::trace_span!("Reading packets", peer=?entity);
        let _entered_span = span.enter();

        // Process all packets
        'h: while let Some(packet) = meta.packet_queue.pop_incoming() {
            let mut reader = Reader::new(packet.payload);

            /*
                Header parsing
            */

            // Get the header bitfield
            let header = u16::from_be_bytes(try_read!(reader.read_array::<2>(), continue 'h));
            let header = PacketHeader::from(header);

            // Reliable packets have extra data
            if header.flagged_reliable() {
                // These two are easy enough
                let sequence = u16::from_be_bytes(try_read!(reader.read_array::<2>(), continue 'h)).into();
                let ack = u16::from_be_bytes(try_read!(reader.read_array::<2>(), continue 'h)).into();

                // Getting the bitfield is more involved
                // since its length is not constant
                let mut arr = [0u8; 16];
                let mut slice = try_read!(reader.read_slice(config.reliable_bitfield_length), continue 'h);
                slice.copy_to_slice(&mut arr[..config.reliable_bitfield_length]);
                let ack_bitfield = u128::from_ne_bytes(arr);

                // Finally, acknowledge the packet
                state.reliability.ack(
                    ReliablePacketHeader { sequence, ack, ack_bitfield },
                    config.reliable_bitfield_length as u8
                );
            }

            /*
                Frame separation
            */

            'r: loop {
                // Check if we're done
                if !reader.has_remaining(6) { break 'r }

                // Try to get the channel value integer
                let channel_int = u32::from_be_bytes(try_read!(reader.read_array::<4>(), break 'r));

                // Get the length of the packet
                let length = u16::from_be_bytes(try_read!(reader.read_array::<2>(), break 'r));

                // Check channel int value
                // This is because 0 is a reserved value for system messages
                // If it's non zero then it's actually been altered, and we need
                // to undo that before passing it to the registry, to ensure good values.
                if channel_int == 0 {
                    todo!()
                } else {
                    // Correct the integer value of the channel
                    let channel_int = channel_int - 1;

                    // Get channel data from registry
                    let channel = ChannelId::from(channel_int);
                    if !registry.channel_exists(channel) { continue 'r; }
                    let channel_data = registry.channel_config(channel).unwrap();

                    // If it has one, get the reliable header.
                    let ordering = match channel_data.ordered {
                        OrderingGuarantee::Unordered => None,
                        _ => {
                            Some(u16::from_be_bytes(try_read!(reader.read_array::<2>(), break 'r)))
                        },
                    };

                    // Finally, get the payload.
                    let payload = try_read!(reader.read_bytes(length.into()), break 'r);

                    match ordering {
                        Some(sequence) => {
                            // Ordered messages are added to a queue

                            // Fetch the queue structure
                            let ordering = state.ordering_entry(channel, || channel_data);

                            // Store in the queue structure
                            let message = ordering.recv(OrderedMessage {
                                sequence: sequence.into(),
                                payload
                            });

                            // Check if it was returned immediately
                            if let Some(message) = message {
                                incoming.push(channel, message.payload);
                            }

                            // TODO: Check if any other values were freed up
                        },
                        None => {
                            // Unordered messages are pushed immediately
                            incoming.push(channel, payload);
                        },
                    }
                }
            }
        }
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
        // Check if there's anything to process
        if outgoing.count() == 0 { return }

        // Fetch or create the thread local scratch space
        let scratch_cell = scratch.0.get_or(|| Cell::new(PacketBuilderSystemScratchInner {
            // These seem like reasonable defaults.
            bytes: BytesMut::with_capacity(MTU_SIZE),
            bins: Vec::with_capacity(16),
            messages: Vec::with_capacity(256),
        }));

        // Tracing info for logging
        let span = tracing::trace_span!("Building packets", peer=?entity);
        let _entered_span = span.enter();

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

        // TODO: Make this return &mut Bin instead of the index
        fn find_or_create_bin(bins: &mut Vec<Bin>, pld_len: usize, header: impl Fn() -> PacketHeader) -> usize {
            // Check if a bin with sufficient remaining space exists
            if let Some((index, _)) = bins.iter_mut()
            .enumerate()
            .find(|(_, bin)| {
                let capacity = bin.buffer.capacity() - BLANK_PREFIX_LENGTH;
                let remaining = capacity - bin.buffer.len();
                remaining >= pld_len
            }) { return index }

            // We couldn't find a bin, so make one
            let mut buffer = Vec::with_capacity(PKT_LEN_WITH_PREFIX);
            buffer.extend_from_slice(&[0u8; BLANK_PREFIX_LENGTH]);
            bins.push(Bin { header: header(), buffer, messages: 0 });
            return bins.len() - 1;
        }

        // Process all messages
        for message in scratch.messages.iter() {
            // Put the channel id into the buffer
            scratch.bytes.put_u32(u32::from(message.channel).wrapping_add(1));

            // Append the payload length
            scratch.bytes.put_u16(u16::try_from(message.payload.len()).unwrap());

            // If present, put ordering data into buffer
            if message.flags.is_ordered() {
                let ordering_data = state.ordering_entry(message.channel, || {
                    registry.channel_config(message.channel).unwrap()
                });

                scratch.bytes.put_u16(ordering_data.advance().into());
            }

            // Put the message payload into the buffer
            scratch.bytes.put(&*message.payload);

            // Use the first-fit algorithm to find a bin
            let bin = find_or_create_bin(&mut scratch.bins, scratch.bytes.len(), || {
                let mut header = PacketHeader::new();
                if message.flags.is_reliable() { header |= PacketHeader::FLAG_RELIABLE; }
                return header;
            });
            let bin = &mut scratch.bins[bin];

            // Extend the bin by the message buffer, and clear the message buffer
            bin.buffer.extend_from_slice(&scratch.bytes);
            bin.messages += 1;
            scratch.bytes.clear();
        }

        // Queue old reliable messages for resending
        let now = Instant::now();
        let reliable_timeout = state.reliable_timeout;
        let mut resends = state.reliability.drain_old(|then| now.duration_since(then) > reliable_timeout);
        while let Some(packet) = resends.next() {
            // Get a bin to pack it into
            let bin = find_or_create_bin(&mut scratch.bins, scratch.bytes.len(), || {
                let mut header = PacketHeader::new();
                header |= PacketHeader::FLAG_RELIABLE; // We know this message is reliable.
                return header;
            });
            let bin = &mut scratch.bins[bin];

            // Add the message to the bin
            bin.buffer.extend_from_slice(&packet.payload);
        }

        // Manual drop to make borrowck happy
        drop(resends);

        // Add bins to the send queue after some tweaks
        for mut bin in scratch.bins.drain(..) {
            // Some variables about the bin
            let is_reliable = bin.header.flagged_reliable();
            let mut sequence = SequenceId::default(); // not relevant until later

            // Append the packet header
            scratch.bytes.put_u16(bin.header.into());

            // If the packet is reliable, append a packet header.
            if is_reliable {
                // Create header
                let header = state.reliability.header();
                state.reliability.advance();
                sequence = header.sequence;

                // Write header integers
                scratch.bytes.put_u16(header.sequence.into());
                scratch.bytes.put_u16(header.ack.into());

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
            let packet = Bytes::from(bin.buffer).slice(offset..total_len);
            let payload = packet.slice(..length);

            // Reliable packets need to be stored until acked
            if is_reliable {
                state.reliability.record(sequence, payload.clone());
            }

            // Finally, put it in the buffer for sending
            meta.packet_queue.push_outgoing(OutgoingPacket {
                payload: packet,
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