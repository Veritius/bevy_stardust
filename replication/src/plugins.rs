//! Plugin groups.

use bevy::{app::PluginGroupBuilder, prelude::*};

/// Adds the default replication plugins.
pub struct ReplicationPlugins;

impl PluginGroup for ReplicationPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(crate::rooms::RoomsPlugin)
            .add(crate::entities::EntityReplicationPlugin)
            .add(crate::hierarchy::HierarchyReplicationPlugin)
    }
}