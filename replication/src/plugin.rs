//! Main plugin for replication.

use bevy::prelude::*;
use bevy_stardust::prelude::*;

/// Adds basic replication functionality.
/// 
/// This must be added:
/// - After the Stardust plugin
/// - Before other replication plugins
pub struct ReplicationPlugin;

impl Plugin for ReplicationPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<StardustPlugin>() {
            panic!("StardustPlugin must be added before ReplicationPlugin")
        }

        todo!();
    }
}