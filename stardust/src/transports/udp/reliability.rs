use std::{collections::BTreeMap, sync::Arc};
use bevy::prelude::*;

/// The reliability state of a [UdpConnection](super::connections::UdpConnection).
#[derive(Component)]
pub(super) struct Reliability {
    /// The local sequence value. Incremented whenever a packet is sent to the peer.
    local: u16,
    /// The remote sequence value. Updated to the most recent sequence ID of packets received from the peer.
    remote: u16,
    /// Packets that have yet to be acknowledged.
    waiting: BTreeMap<u16, Arc<[u8]>>,
    /// Estimate of how much space `waiting` is taking up.
    using_bytes: usize,
}

impl Reliability {
    /// Returns an estimate of how much memory unacknowledged messages are taking up, in bytes.
    /// Does not take into account the size of the map and pointers.
    pub fn waiting(&self) -> usize {
        self.using_bytes
    }

    /// Returns a sequence ID for an outgoing packet, incrementing the local counter.
    pub fn increment_local(&mut self) -> u16 {
        let old = self.local;
        self.local = self.local.wrapping_add(1);
        return old
    }

    pub fn update_remote(&mut self, seq: u16) {
        todo!()
    }
}

impl Default for Reliability {
    fn default() -> Self {
        Self {
            local: 0,
            remote: 0,
            waiting: BTreeMap::new(),
            using_bytes: 0,
        }
    }
}

impl std::fmt::Debug for Reliability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Reliability")
        .field("local", &self.local)
        .field("remote", &self.remote)
        .field("waiting", &format!("{} entries", &self.waiting.len()))
        .field("using_bytes", &self.using_bytes)
        .finish()
    }
}

/// Returns `true` if `u1` is greater than `u2` while considering sequence id wrap-around.
/// 
/// Based on [Glenn Fiedler's article](https://gafferongames.com/post/reliability_ordering_and_congestion_avoidance_over_udp/) on reliability.
#[inline]
fn sequence_greater_than(u1: u16, u2: u16) -> bool {
    ( (u1 > u2) && (u1 - u2 <= 32768) ) ||
    ( (u1 < u2) && (u2 - u1 >  32768) )
}