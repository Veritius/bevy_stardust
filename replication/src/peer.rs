use bevy::prelude::*;
use bevy_stardust::prelude::*;

/// Stardust channel for negotiating replication peer data.
#[derive(Default)]
pub(crate) struct PeerNegotiation;

/// Added to [`NetworkPeer`] entities that are replicating data.
#[derive(Debug, Component, Reflect)]
pub struct ReplicationPeer {
    pub(crate) side: Option<Side>,
}

impl Default for ReplicationPeer {
    fn default() -> Self {
        Self {
            side: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub(crate) enum Side { Left, Right }