use std::{collections::BTreeMap, fmt::Debug};
use crate::prelude::{ChannelId, OctetString};

/// Ordering data for a single peer.
#[derive(Debug)]
pub(super) struct Ordering {
    /// Ordering data of connection management messages.
    /// Mainly used during the handshake.
    pub main: OrderingData,
    /// Ordering data of individual channels.
    channels: BTreeMap<ChannelId, OrderingData>
}

impl Default for Ordering {
    fn default() -> Self {
        Self {
            main: OrderingData::default(),
            channels: Default::default()
        }
    }
}

/// Ordering data for a single channel.
pub(super) struct OrderingData {
    using_bytes: usize,
    latest: u16,
    pending: BTreeMap<u16, OctetString>,
}

impl OrderingData {
    /// Returns an estimate of how much memory that messages stuck in the queue are using, in bytes.
    /// Does not take into account the size of the map and pointers.
    pub fn waiting(&self) -> usize {
        self.using_bytes
    }
}

impl Default for OrderingData {
    fn default() -> Self {
        Self {
            using_bytes: 0,
            latest: 0,
            pending: BTreeMap::new(),
        }
    }
}

impl Debug for OrderingData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChannelOrdering").finish()
    }
}

/// Returns `true` if `u1` is greater than `u2` while considering sequence id wrap-around.
/// 
/// Based on [Glenn Fiedler's article](https://gafferongames.com/post/reliability_ordering_and_congestion_avoidance_over_udp/) on reliability.
#[inline]
pub(super) fn sequence_greater_than(u1: u16, u2: u16) -> bool {
    ( (u1 > u2) && (u1 - u2 <= 32768) ) ||
    ( (u1 < u2) && (u2 - u1 >  32768) )
}