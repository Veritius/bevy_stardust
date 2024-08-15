//! Plugin groups.

use bevy::{app::PluginGroupBuilder, prelude::*};
use crate::config::Clusivity;

/// Adds the default replication plugins.
pub struct ReplicationPlugins;

impl PluginGroup for ReplicationPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(crate::rooms::RoomsPlugin)
            .add(crate::entities::EntityReplicationPlugin {
                opt: Clusivity::Include,
            })
    }
}