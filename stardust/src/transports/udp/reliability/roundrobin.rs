//! Sequence-based reliability using round robin negative acknowledgement.

use std::marker::PhantomData;
use super::bits::{SequenceNumber, SequenceBitset};

/// State information for round-robin reliability.
/// 
/// See the [module level documentation](self).
pub struct RoundRobinReliabilityData<I: SequenceNumber, B: SequenceBitset> {
    pub local: I,
    pub remote: I,
    phantom: PhantomData<B>,
}