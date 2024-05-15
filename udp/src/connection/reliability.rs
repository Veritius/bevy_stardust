use std::{cmp::Ordering, fmt::Debug, time::Instant};
use bytes::Bytes;
use crate::sequences::*;

#[derive(Debug, Clone)]
pub(crate) struct ReliabilityState {
    pub local_sequence: SequenceId,
    pub remote_sequence: SequenceId,
    pub ack_memory: AckMemory,
}

impl ReliabilityState {
    pub fn new() -> Self {
        Self {
            local_sequence: SequenceId::random(),
            remote_sequence: 0.into(),
            ack_memory: AckMemory::default(),
        }
    }

    /// Increments the local sequence by 1
    pub fn advance(&mut self) {
        self.local_sequence += 1;
    }

    /// Acknowledge a remote packet's sequence id.
    pub fn ack_seq(&mut self, seq: SequenceId) {
        // Update bitfield and remote sequence
        let diff = seq.wrapping_diff(&self.remote_sequence);
        match self.remote_sequence.cmp(&seq) {
            Ordering::Greater => {
                // The packet is older, flag it as acknowledged
                self.ack_memory.set_high(diff as u32);
            },
            Ordering::Less => {
                // The packet is newer, shift the memory bitfield
                self.remote_sequence = seq;
                self.ack_memory.shift(diff as u32);
                self.ack_memory.set_high(diff.wrapping_sub(1) as u32);
            },
            Ordering::Equal => {}, // Shouldn't happen.
        }
    }

    /// Record a remote packet's acknowledgements.
    pub fn rec_ack(
        &mut self,
        ack: SequenceId,
        bitfield: AckMemory,
        bitfield_bytes: u8
    ) -> impl Iterator<Item = SequenceId> + Clone {
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
                    let mask = AckMemory::BITMASK.overflowing_shl(self.cursor.into()).0;
                    if self.bitfield & mask == 0 { self.cursor += 1; continue }
                    let ack = self.origin + self.cursor as u16;

                    // Success, advance cursor and return
                    self.cursor += 1;
                    return Some(ack.into())
                }
            }
        }

        // Create iterator
        AcknowledgementIterator {
            origin: ack,
            cursor: 0,
            limit: (bitfield_bytes * 8),
            bitfield: bitfield.0,
        }
    }
}

#[derive(Default, Clone)]
pub struct AckMemory(u128);

impl AckMemory {
    const BITMASK: u128 = 1;

    #[inline]
    pub fn from_array(array: [u8; 16]) -> Self {
        Self(u128::from_be_bytes(array))
    }

    pub fn from_slice(slice: &[u8]) -> Result<Self, ()> {
        let len = slice.len().min(16);
        if len == 0 { return Err(()); }
        let mut bytes = [0u8; 16];
        bytes[..len].copy_from_slice(&slice[..len]);
        return Ok(Self::from_array(bytes))
    }

    #[inline]
    pub fn into_array(&self) -> [u8; 16] {
        self.0.to_be_bytes()
    }

    pub fn into_u16(&self) -> u16 {
        let a = self.into_array();
        u16::from_be_bytes([a[0], a[1]])
    }

    pub fn from_u16(value: u16) -> Self {
        let a = value.to_be_bytes();
        let mut b = [0u8; 16];
        b[0] = a[0]; b[1] = a[1];
        Self(u128::from_be_bytes(b))
    }

    #[inline]
    pub fn shift(&mut self, bits: u32) {
        self.0 = self.0.overflowing_shl(bits).0
    }

    #[inline]
    pub fn set_high(&mut self, idx: u32) {
        self.0 |= Self::BITMASK.overflowing_shl(idx).0;
    }
}

impl Debug for AckMemory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:b}", self.0))
    }
}

#[derive(Debug)]
pub(crate) struct UnackedPacket {
    pub payload: Bytes,
    pub time: Instant,
}