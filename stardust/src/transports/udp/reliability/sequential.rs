//! Sequence-based reliability using bitflags, similar to how TCP works.
//! Based on [Gaffer on Games' article] on the subject.
//! 
//! [Gaffer on Games' article]: https://www.gafferongames.com/post/reliability_ordering_and_congestion_avoidance_over_udp/

use std::marker::PhantomData;
use super::bits::{SequenceNumber, SequenceBitset};

mod private {
    pub trait Sealed {}
    impl Sealed for u8 {}
    impl Sealed for u16 {}
    impl Sealed for u32 {}
}

/// State information for sequential reliability.
pub struct SequentialReliabilityData<I: SequenceNumber, B: SequenceBitset> {
    pub local: I,
    pub remote: I,
    phantom: PhantomData<B>,
}