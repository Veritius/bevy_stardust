use bevy::prelude::*;
use crate::prelude::*;

/// Enables replicating entity hierarchies.
pub struct HierarchyReplicationPlugin;

impl Plugin for HierarchyReplicationPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<EntityReplicationPlugin>() {
            panic!("HierarchyReplicationPlugin must be added after EntityReplicationPlugin");
        }

        app.register_type::<ReplicateHierarchy>();
    }
}

/// Automatically replicates entities that are descendants of entities with this component.
#[derive(Debug, Default, Component, Clone, Copy, Reflect)]
#[reflect(Debug, Default, Component)]
pub struct ReplicateHierarchy;