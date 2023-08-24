//! Shared plugin for Stardust.

use bevy::prelude::*;
use crate::{server::plugin::StardustServerPlugin, client::plugin::StardustClientPlugin};
use super::{scheduling::{network_pre_update, network_post_update, add_schedules}, channels::systems::*, hashdiff::{UniqueNetworkHasher, complete_hasher}, prelude::*};

/// Shared information between the client and server.
pub struct StardustSharedPlugin;
impl Plugin for StardustSharedPlugin {
    fn build(&self, app: &mut App) {
        add_schedules(app);

        app.add_systems(PreStartup, complete_hasher);
        
        app.add_systems(PreUpdate, network_pre_update);
        app.add_systems(PostUpdate, network_post_update);

        app.add_systems(PreUpdate, panic_on_channel_removal);

        // Systems for clearing the buffers
        app.add_systems(NetworkPreUpdateCleanup, clear_incoming_buffers_system);
        app.add_systems(NetworkPostUpdateCleanup, clear_outgoing_buffers_system);

        app.insert_resource(ChannelRegistry::new());
        app.insert_resource(UniqueNetworkHasher::new());
    }

    fn finish(&self, app: &mut App) {
        if app.is_plugin_added::<StardustServerPlugin>() && app.is_plugin_added::<StardustClientPlugin>() {
            panic!("You can't be both a client and a server!");
        }
    }
}