use bevy::prelude::*;
use crate::shared::integers::u24;

pub type SequenceId = u24;

/// Sequence information for one client.
/// 
/// Based on [this post by Gaffer on Games](https://gafferongames.com/post/reliability_ordering_and_congestion_avoidance_over_udp/).
#[derive(Debug, Component)]
pub struct ClientSequenceData {
    pub local_sequence: SequenceId,
    pub remote_sequence: SequenceId,
}

impl ClientSequenceData {
    pub fn new() -> Self {
        Self {
            local_sequence: 0.into(),
            remote_sequence: 0.into(),
        }
    }
    
    /// Returns the next sequence ID, advancing the local sequence value.
    pub fn next(&mut self) -> SequenceId {
        let new = self.local_sequence.wrapping_add(1.into());
        self.local_sequence = new;
        return new;
    }

    /// Returns a bitfield of acknowledgements.
    pub fn acks(&self) -> u32 {
        todo!();
        0
    }

    /// Checks if `new` is greater than the current remote value, replacing it if true.
    pub fn update_remote(&mut self, new: SequenceId) {
        if sequence_more_recent_than(self.remote_sequence, new) {
            self.remote_sequence = new;
        }
    }
}

/// Returns if a sequence ID is more recent than another.
/// 
/// Based on [this post by Gaffer on Games](https://gafferongames.com/post/reliability_ordering_and_congestion_avoidance_over_udp/).
pub fn sequence_more_recent_than(left: SequenceId, right: SequenceId) -> bool {
    let middle = u24::try_from(2u32.pow(23)).unwrap();
    ( ( left > right ) && ( left - right <= middle ) ) ||
    ( ( left < right ) && ( right - left > middle ) )
}