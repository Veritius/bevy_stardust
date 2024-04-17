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

/// Automatically replicates entities in the hierarchy.
#[derive(Debug, Default, Component, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
#[reflect(Debug, Default, Component, PartialEq, Hash)]
pub enum ReplicateHierarchy {
    /// Replicates all children, adding all components necessary to do so.
    Enabled,

    /// Doesn't automatically replicate children.
    /// Entities can still be replicated by adding components manually.
    Disabled,

    /// Inherit the replication mode from a parent, if any.
    /// Defaults to [`Enabled`](ReplicateHierarchy::Enabled) if no parent exists.
    #[default]
    Inherit,
}