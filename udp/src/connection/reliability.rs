use std::{cmp::Ordering, collections::BTreeMap, time::Instant};
use bytes::Bytes;
use crate::sequences::*;

const BITMASK: u128 = 1 << 127;

#[derive(Debug, Clone)]
pub(crate) struct ReliabilityState {
    pub local_sequence: SequenceId,
    pub remote_sequence: SequenceId,
    pub sequence_memory: u128,
}

impl ReliabilityState {
    pub fn new() -> Self {
        Self {
            local_sequence: SequenceId::random(),
            remote_sequence: 0.into(),
            sequence_memory: 0,
        }
    }

    /// Creates a ReliablePacketHeader for your outgoing data.
    pub fn header(&self) -> ReliablePacketHeader {
        ReliablePacketHeader {
            sequence: self.local_sequence,
            ack: self.remote_sequence,
            ack_bitfield: self.sequence_memory
        }
    }

    /// Increments the local sequence by 1
    pub fn increment_local(&mut self) {
        self.local_sequence += 1;
    }

    /// Acknowledge packets identified in a reliable header. Returns an iterator over the sequences of packets that have been acknowledged by the remote peer.
    pub fn ack(&mut self, header: ReliablePacketHeader, bitfield_bytes: u8) -> impl Iterator<Item = SequenceId> + Clone {
        // Update bitfield and remote sequence
        let diff = header.sequence.diff(&self.remote_sequence);
        match header.sequence.cmp(&self.remote_sequence) {
            Ordering::Greater => {
                // Newer packet, shift the memory bitfield
                self.remote_sequence = header.sequence;
                self.sequence_memory = self.sequence_memory.overflowing_shr(diff.into()).0;
            },
            _ => {
                // Older packet, mark id as acknowledged
                self.sequence_memory |= BITMASK.overflowing_shr(diff.into()).0;
            }
        }

        // Iterator object for acknowledgements
        #[derive(Clone)]
        struct AcknowledgementIterator {
            origin: SequenceId,
            cursor: u8,
            limit: u8,
            bitfield: u128,
        }

        impl Iterator for AcknowledgementIterator {
            type Item = SequenceId;
        
            fn next(&mut self) -> Option<Self::Item> {
                // Get the ack value
                loop {
                    // Check if we've reached the limit
                    if self.cursor == self.limit { return None }

                    // Get the ack value
                    let mask = BITMASK >> self.cursor;
                    if self.bitfield & mask == 0 { self.cursor += 1; continue }
                    let ack = self.origin - self.cursor as u16;

                    // Success, advance cursor and return
                    self.cursor += 1;
                    return Some(ack.into())
                }
            }
        }

        // Create iterator
        AcknowledgementIterator {
            origin: header.ack,
            cursor: 0,
            limit: (bitfield_bytes * 8),
            bitfield: header.ack_bitfield
        }
    }
}

/// Required information for a reliable packet.
#[derive(Debug, Clone, Copy)]
pub struct ReliablePacketHeader {
    pub sequence: SequenceId,
    pub ack: SequenceId,
    pub ack_bitfield: u128,
}

pub(crate) struct ReliablePackets {
    pub state: ReliabilityState,
    unacked: BTreeMap<u16, UnackedPacket>,
}

impl ReliablePackets {
    pub fn new(state: ReliabilityState) -> Self {
        Self {
            unacked: BTreeMap::default(),
            state,
        }
    }

    #[inline]
    pub fn header(&self) -> ReliablePacketHeader {
        self.state.header()
    }

    #[inline]
    pub fn increment_local(&mut self) {
        self.state.increment_local()
    }

    pub fn record(&mut self, sequence: u16, payload: Bytes) {
        self.unacked.insert(sequence, UnackedPacket {
            payload,
            time: Instant::now(),
        });
    }

    pub fn ack(&mut self, header: ReliablePacketHeader, bitfield_bytes: u8) {
        // Update reliability state
        let iter = self.state.ack(header, bitfield_bytes);

        // Remove all acked packets from storage
        for seq in iter {
            self.unacked.remove(&seq.into());
        }
    }

    pub fn drain_old<'a, Filter: Fn(Instant) -> bool + 'a>(&'a mut self, filter: Filter) -> impl Iterator<Item = UnackedPacket> + 'a {
        // TODO: When btree_extract_if is stabilised, use that instead.
        struct FilterTaker<'a, Filter>(&'a mut ReliablePackets, Filter);
        impl<'a, Filter: Fn(Instant) -> bool> Iterator for FilterTaker<'a, Filter> {
            type Item = UnackedPacket;
        
            fn next(&mut self) -> Option<Self::Item> {
                // Try to find a key
                let key = self.0.unacked.iter()
                    .filter(|(_, v)| { (self.1)(v.time) })
                    .map(|(k, _)| *k)
                    .next()?;
                
                // Take the packet from the map and return it
                return self.0.unacked.remove(&key);
            }
        }

        // Return the iterator
        return FilterTaker(self, filter);
    }
}

pub(crate) struct UnackedPacket {
    payload: Bytes,
    time: Instant,
}