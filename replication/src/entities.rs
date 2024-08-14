//! Entity replication.

use bevy::prelude::*;
use crate::identifiers::NetId;

/// Adds functionality for replicating entities.
pub struct EntityReplicationPlugin;

impl Plugin for EntityReplicationPlugin {
    fn build(&self, app: &mut App) {

    }
}

/// Attached to entities to replicate them over the network.
#[derive(Debug, Component)]
pub struct Replicated {
    id: NetId,
}