use bevy::prelude::*;

/// The reliability state of an [EstablishedUdpPeer](super::connections::EstablishedUdpPeer)
#[derive(Debug, Component)]
pub struct Reliability {
    /// The local sequence value. Incremented whenever a packet is sent to the peer.
    pub local: u16,
    /// The remote sequence value. Updated to the most recent sequence ID of packets received from the peer.
    pub remote: u16,
}

impl Default for Reliability {
    fn default() -> Self {
        Self {
            local: 0,
            remote: 0,
        }
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