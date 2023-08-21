//! Negative acknowledgement based reliability.
//! Functions based on the assumption a lot of messages are being sent every frame.

use bevy::prelude::*;
use fixedbitset::FixedBitSet;
use super::integers::u24;

pub type SequenceId = u24;

/// Returns if a sequence ID is more recent than another.
/// 
/// Based on [this post by Gaffer on Games](https://gafferongames.com/post/reliability_ordering_and_congestion_avoidance_over_udp/).
pub fn sequence_more_recent_than(left: SequenceId, right: SequenceId) -> bool {
    let middle = u24::try_from(2u32.pow(23)).unwrap();
    ( ( left > right ) && ( left - right <= middle ) ) ||
    ( ( left < right ) && ( right - left > middle ) )
}

#[derive(Debug, Component)]
pub struct PeerSequenceData {
    pub local_sequence: SequenceId,
    pub remote_sequence: SequenceId,
    highest_remote_sequence_this_frame: SequenceId,
    received: FixedBitSet,
}

impl PeerSequenceData {
    pub fn new() -> Self {
        // I reckon 4096 reliable packets each tick is probably the most that'll happen.
        let received = FixedBitSet::with_capacity(4096);

        Self {
            local_sequence: 0.into(),
            remote_sequence: 0.into(),
            highest_remote_sequence_this_frame: 0.into(),
            received,
        }
    }

    /// Returns the next sequence ID, advancing the local sequence value.
    pub fn next(&mut self) -> SequenceId {
        let new = self.local_sequence.wrapping_add(1.into());
        self.local_sequence = new;
        dbg!(&self.received);
        return new;
    }

    /// Sets the highest remote sequence value for this frame if it's larger.
    pub fn set_remote_sequence(&mut self, sequence: SequenceId) {
        if sequence_more_recent_than(self.highest_remote_sequence_this_frame, sequence) { return; }
        self.highest_remote_sequence_this_frame = sequence;
    }

    /// Marks a packet as received. Expands the bitvec if it's too small.
    pub fn mark_received(&mut self, sequence: SequenceId) {
        let seq: usize = sequence.wrapping_sub(self.remote_sequence).into();
        if seq > self.received.len() {
            self.received.grow(seq);
        }
        self.received.insert(seq);
    }

    /// Run after all packets from this client are read.
    /// Updates `remote_sequence` and returns an iterator over all missing packet IDs if there are any.
    pub fn complete(&mut self) -> Option<impl Iterator<Item = SequenceId>> {
        // Iterator definition
        struct MissingPacketIterator {
            highest: usize,
            index: usize,
            sequence: SequenceId,
            bits: FixedBitSet,
        }

        impl Iterator for MissingPacketIterator {
            type Item = SequenceId;

            fn next(&mut self) -> Option<Self::Item> {
                // Read through the bitvec until we run out of items or find the next false value
                loop {
                    // Return None if we've read everything
                    if self.index > self.highest { break None; }

                    // Get the value and increment index
                    let v = self.bits.contains(self.index);

                    // Next iteration of loop
                    if v == true {
                        self.index += 1;
                        continue;
                    }

                    // Figure out the sequence ID
                    let val = self.sequence.wrapping_add(self.index.try_into().unwrap());
                    self.index += 1;
                    break Some(val)
                }
            }
        }

        // Clone remote sequence ID for iterator
        let cseq = self.remote_sequence.clone();

        // Update remote sequence value
        let recv_amt = self.highest_remote_sequence_this_frame.wrapping_sub(self.remote_sequence).into();
        self.remote_sequence = self.highest_remote_sequence_this_frame;

        // Check if any packets weren't received
        if self.received.is_clear() { return None; }
        
        // Build iterator, cloning data
        let iterator = MissingPacketIterator {
            highest: recv_amt,
            index: 0,
            sequence: cseq,
            bits: self.received.clone(),
        };

        // Clear bit vector
        self.received.clear();

        // Return iterator
        Some(iterator)
    }
}