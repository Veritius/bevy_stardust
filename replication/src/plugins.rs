//! Plugin groups.

use bevy::{app::PluginGroupBuilder, prelude::*};
use crate::config::ReplicateOpt;

/// Adds the default replication plugins.
pub struct ReplicationPlugins;

impl PluginGroup for ReplicationPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(crate::entities::EntityReplicationPlugin {
                opt: ReplicateOpt::Out,
            })
    }
}