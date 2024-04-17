use bevy::prelude::*;

/// Stardust channel for negotiating replication peer data.
#[derive(Default)]
pub(crate) struct PeerNegotiation;

/// Added to [`NetworkPeer`] entities that are replicating data.
#[derive(Debug, Component, Reflect)]
#[reflect(Debug, Component)]
pub struct ReplicationPeer {
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

}

impl Default for ReplicationPeerInner {
    fn default() -> Self {
        Self {

        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Side { Left, Right }