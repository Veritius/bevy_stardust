//! Negative acknowledgement based reliability.
//! Functions based on the assumption a lot of messages are being sent every frame.

use bevy::prelude::*;
use fixedbitset::FixedBitSet;

pub const MAX_RELIABLE_MESSAGES_PER_CYCLE: usize = 2usize.pow(16);

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SequenceId(u16);

impl SequenceId {
    pub const HALF: Self = Self(32768);

    #[inline]
    pub fn wrapping_add(self, rhs: Self) -> Self {
        Self(self.0.wrapping_add(rhs.0))
    }

    #[inline]
    pub fn wrapping_sub(self, rhs: Self) -> Self {
        Self(self.0.wrapping_sub(rhs.0))
    }

    #[inline]
    pub fn bytes(&self) -> [u8; 2] {
        self.0.to_be_bytes()
    }
}

impl std::fmt::Debug for SequenceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl PartialOrd for SequenceId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.0 == other.0 { return Some(std::cmp::Ordering::Equal); }
        let comp =
            ((self.0 > other.0) && (self.0 - other.0 < Self::HALF.into())) ||
            ((self.0 < other.0) && (other.0 - self.0 > Self::HALF.into()));
        match comp {
            true => Some(std::cmp::Ordering::Greater),
            false => Some(std::cmp::Ordering::Less),
        }
    }
}

impl Ord for SequenceId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl From<[u8; 2]> for SequenceId {
    fn from(value: [u8; 2]) -> Self {
        Self(u16::from_be_bytes(value))
    }
}

impl From<u16> for SequenceId {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl Into<u16> for SequenceId {
    fn into(self) -> u16 {
        self.0
    }
}

impl Into<usize> for SequenceId {
    fn into(self) -> usize {
        self.0 as usize
    }
}

/// Stores sequence data for reliability functionality.
#[derive(Debug, Component)]
pub struct PeerSequenceData {
    pub local_sequence: SequenceId,
    pub remote_sequence: SequenceId,
    pub cycle_latest_remote: SequenceId,
    bitstore: FixedBitSet,
}

impl PeerSequenceData {
    pub fn new() -> Self {
        Self {
            local_sequence: 0.into(),
            remote_sequence: 0.into(),
            cycle_latest_remote: 0.into(),
            // 2048 reliable messages in one cycle is probably the most that'll happen.
            bitstore: FixedBitSet::with_capacity(2048),
        }
    }

    /// Returns the next `SequenceId` for use in sending packets.
    pub fn next(&mut self) -> SequenceId {
        let sequence = self.local_sequence.clone();
        self.local_sequence = self.local_sequence.wrapping_add(1.into());
        sequence
    }

    /// If `highest` is greater than `self.frame_remote`, sets `self.frame_remote` to `highest`.
    pub fn try_highest(&mut self, highest: SequenceId) {
        if highest > self.cycle_latest_remote {
            self.cycle_latest_remote = highest;
        }
    }

    /// Marks a packet as received using its sequence ID. Resizes the storage vector if it's too small.
    pub fn mark_received(&mut self, sequence: SequenceId) {
        let idx: usize = sequence.wrapping_sub(self.remote_sequence).into();
        if idx >= self.bitstore.len() { self.bitstore.grow(idx); }
        self.bitstore.set(idx, true);
    }

    /// Complete one reliability cycle, returning an iterator of all missed packets and updating `remote_sequence`.
    pub fn complete_cycle(&mut self) -> impl Iterator<Item = SequenceId> {
        // Get statistics values
        let difference = self.cycle_latest_remote.wrapping_sub(self.remote_sequence);

        // Construct iterator
        let iter = MissedPacketIterator {
            bitstore: self.bitstore.clone(),
            offset: self.remote_sequence,
            index: 0,
            highest: difference.into(),
        };

        // Reset state for the next cycle
        self.remote_sequence = self.cycle_latest_remote;
        self.bitstore.clear();

        // Return iterator
        iter
    }
}

struct MissedPacketIterator {
    bitstore: FixedBitSet,
    offset: SequenceId,
    index: u16,
    highest: u16,
}

impl Iterator for MissedPacketIterator {
    type Item = SequenceId;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.index == self.highest { return None; }

            if self.bitstore[self.index as usize] == true {
                self.index += 1;
                continue;
            }

            let true_seq = self.offset.wrapping_add(self.index.into());
            self.index += 1;
            return Some(true_seq)
        }
    }
}