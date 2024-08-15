//! Configuration for peers that use replication.

use bevy::prelude::*;
use crate::identifiers::*;

/// A component attached to [peers](bevy_stardust::connections::Peer) to enable replication.
#[derive(Debug, Component)]
pub struct ReplicationPeer {
    side: Side,
}

impl ReplicationPeer {
    /// Creates a new [`ReplicationPeer`] component.
    pub fn new(
        side: Side,
    ) -> Self {
        Self {
            side,
        }
    }

    /// Returns the [`Side`] of the peer.
    pub fn side(&self) -> Side {
        self.side
    }
}