//! Sequence-based reliability using bitflags, similar to how TCP works.
//! Based on [Gaffer on Games' article] on the subject.
//! 
//! [Gaffer on Games' article]: https://www.gafferongames.com/post/reliability_ordering_and_congestion_avoidance_over_udp/

use std::cmp::Ordering;
use super::bits::{SequenceNumber, SequenceBitset};

/// State information for sequential reliability.
/// 
/// See the [module level documentation](self).
pub struct SequentialReliabilityData<I: SequenceNumber, B: SequenceBitset> {
    pub local: I,
    pub remote: I,
    pub bitset: B,
}

impl<I: SequenceNumber, B: SequenceBitset> SequentialReliabilityData<I, B> {
    /// Increments the local counter by one.
    /// This function should be called every time a packet is sent.
    pub fn on_send(&mut self) {
        self.local = self.local.wrapping_add(I::VAL_ONE);
    }

    /// Updates `remote` and `bitset` based on the received sequence value.
    /// This function should be called when a packet is received.
    pub fn on_recv(&mut self, sequence: I) {
        if sequence.wrapping_compare(self.remote) == Ordering::Greater {
            let diff = sequence.wrapping_difference(self.remote).to_u8();
            self.bitset.shift_left(diff);
            self.bitset.set_bit_on(0);
            self.remote = sequence;
        }
    }
}