//! Plugin groups.

use bevy::{app::PluginGroupBuilder, prelude::*};
use crate::config::ReplicateOpt;

/// Adds the default replication plugins.
pub struct ReplicationPlugins {
    /// Whether or not to replicate entities by default.
    pub entity_opt: ReplicateOpt,
}

impl PluginGroup for ReplicationPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(crate::entities::EntityReplicationPlugin {
                opt: self.entity_opt,
            })
    }
}