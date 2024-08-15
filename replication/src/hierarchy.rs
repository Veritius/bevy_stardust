//! Replication of entity hierarchies.

use bevy::prelude::*;

/// Adds functionality for replicating entity hierarchies.
pub struct HierarchyReplicationPlugin;

impl Plugin for HierarchyReplicationPlugin {
    fn build(&self, app: &mut App) {

    }
}


/// When added to an entity, it and all its children are set to be replicated.
/// Any children added after the fact will also be set to be replicated.
#[derive(Debug, Component)]
pub struct ReplicateDescendants;
