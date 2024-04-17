use bevy::prelude::*;

/// Stardust channel for negotiating replication peer data.
#[derive(Default)]
pub(crate) struct PeerNegotiation;

/// Automatically added to [`NetworkPeer`] entities that are replicating data.
#[derive(Debug, Component, Reflect)]
#[reflect(Debug, Default, Component)]
pub struct ReplicationPeer {
    // Separate type so non-config options aren't reflected and cannot be mutated
    // ie. changing the side, which would break entity ids, or something else.
    #[reflect(ignore)]
    pub(crate) inner: ReplicationPeerInner,
}

impl Default for ReplicationPeer {
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct ReplicationPeerInner {
    pub side: Option<Side>,
}

impl Default for ReplicationPeerInner {
    fn default() -> Self {
        Self {
            side: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Side { Left, Right }