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