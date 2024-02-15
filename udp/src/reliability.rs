use std::{collections::BTreeMap, time::Instant};
use bytes::Bytes;
use untrusted::{Reader, EndOfInput};
use crate::sequences::*;

const BITMASK: u128 = 1 << 127;

struct SentPacket {
    payload: Bytes,
    time: Instant,
}

pub(crate) struct ReliablePacket {
    local_sequence: u16,
    remote_sequence: u16,
    unacked_packets: BTreeMap<u16, SentPacket>,
    sequence_memory: u128,
}

impl ReliablePacket {
    /// Gets the header of a reliable packet.
    pub fn get_header(reader: &mut Reader<'_>, bitfield_bytes: usize) -> Result<ReliablePacketHeader, EndOfInput> {
        let sequence = u16::from_be_bytes([
           reader.read_byte()?,
           reader.read_byte()?, 
        ]);

        let ack = u16::from_be_bytes([
            reader.read_byte()?,
            reader.read_byte()?,
        ]);

        let mut ack_bitfield_bytes = [0u8; 16];
        ack_bitfield_bytes[..bitfield_bytes].clone_from_slice(
            reader.read_bytes(bitfield_bytes)?.as_slice_less_safe());
        let ack_bitfield = u128::from_ne_bytes(ack_bitfield_bytes);

        Ok(ReliablePacketHeader { sequence, ack, ack_bitfield })
    }

    /// Acknowledge packets identified in a reliable header.
    pub fn acknowledge(&mut self, header: ReliablePacketHeader, bitfield_bytes: usize) {
        // Update bitfield and remote sequence
        let seq_diff = wrapping_diff(header.sequence, self.remote_sequence);
        if sequence_greater_than(header.sequence, self.remote_sequence) {
            // Newer packet, shift the memory bitfield
            self.remote_sequence = header.sequence;
            self.sequence_memory >>= seq_diff;
        } else {
            // Older packet, mark id as acknowledged
            self.sequence_memory |= BITMASK >> seq_diff;
        }

        // Acknowledge the packet identified by the ack seq id
        self.unacked_packets.remove(&header.ack);

        // Acknowledge all sequences in the ack bitfield
        for idx in 0..(bitfield_bytes * 8) {
            let mask = BITMASK >> idx;
            if header.ack_bitfield & mask == 0 { continue }
            let ack = header.ack.wrapping_sub(idx as u16);
            self.unacked_packets.remove(&ack);
        }
    }

    pub fn handle_outgoing(&mut self, payload: Bytes, bitfield_bytes: usize) -> ReliablePacketHeader {
        // Generate header
        let header = ReliablePacketHeader {
            sequence: self.local_sequence,
            ack: self.remote_sequence,
            ack_bitfield: self.sequence_memory
        };

        // Add to unacked packets list
        self.unacked_packets.insert(header.sequence, SentPacket { payload, time: Instant::now() });

        // Move up local sequence counter
        self.local_sequence = self.local_sequence.wrapping_add(1);

        header
    }

    /// Returns the packet corresponding to `id` if it hasn't been acknowledged yet.
    pub fn get_unacked_packet(&self, id: u16) -> Option<Bytes> {
        if let Some(packet) = self.unacked_packets.get(&id) {
            return Some(packet.payload.clone())
        } else { 
            return None ;
        }
    }
}

pub struct ReliablePacketHeader {
    pub sequence: u16,
    pub ack: u16,
    pub ack_bitfield: u128,
}