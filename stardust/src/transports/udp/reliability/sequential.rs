//! Sequence-based reliability using bitflags, similar to how TCP works.
//! Based on [Gaffer on Games' article] on the subject.
//! 
//! [Gaffer on Games' article]: https://www.gafferongames.com/post/reliability_ordering_and_congestion_avoidance_over_udp/

use std::cmp::Ordering;
use super::bits::{SequenceNumber, SequenceBitset, SemiByteArray};

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
        let diff = sequence.wrapping_difference(self.remote).to_u8();

        if sequence.wrapping_compare(self.remote) == Ordering::Greater {
            // Shift bits and set first to on
            self.bitset.shift_left(diff);
            self.bitset.set_bit_on(0);
            self.remote = sequence;
        } else {
            // Set bits normally
            self.bitset.set_bit_on(diff);
        }
    }

    pub fn header(&self) -> SemiByteArray {
        let mut sbi = SemiByteArray::new();
        let bytes_a = self.remote.to_bytes();
        let bytes_b = self.bitset.to_bytes();
        for (i, x) in bytes_a.read().iter().chain(bytes_b.read().iter()).enumerate() {
            sbi.0 = i;
            sbi.1[i] = *x;
        }
        sbi
    }
}