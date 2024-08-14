//! Entity replication.

use bevy::prelude::*;
use crate::{config::ReplicateOpt, identifiers::NetId};

/// Adds functionality for replicating entities.
pub struct EntityReplicationPlugin {
    /// Whether or not to replicate entities by default.
    pub opt: ReplicateOpt,
}

impl Plugin for EntityReplicationPlugin {
    fn build(&self, app: &mut App) {

    }
}

/// Attached to entities to replicate them over the network.
#[derive(Debug, Component)]
pub struct Replicated {
    id: NetId,
}