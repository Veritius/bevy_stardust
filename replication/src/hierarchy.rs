//! Replication of entity hierarchies.

use std::any::type_name;
use bevy::prelude::*;
use crate::entities::EntityReplicationPlugin;

/// Adds functionality for replicating entity hierarchies.
/// 
/// Requires [`EntityReplicationPlugin`] to be added beforehand.
pub struct HierarchyReplicationPlugin;

impl Plugin for HierarchyReplicationPlugin {
    fn build(&self, app: &mut App) {
        assert!(app.is_plugin_added::<crate::entities::EntityReplicationPlugin>(),
            "{} requires {}, but it was not added", type_name::<Self>(), type_name::<EntityReplicationPlugin>());
    }
}

/// When added to an entity, it and all its children are set to be replicated.
/// Any children added after the fact will also be set to be replicated.
#[derive(Debug, Component)]
pub struct ReplicateDescendants;