use std::collections::hash_map::DefaultHasher;

use bevy::prelude::*;
use crate::{server::plugin::StardustServerPlugin, client::plugin::StardustClientPlugin};
use super::{scheduling::{network_pre_update, network_post_update}, channels::systems::panic_on_channel_removal, hashdiff::{UniqueNetworkHasher, UniqueNetworkHash}};

/// Shared information between the client and server.
pub struct StardustSharedPlugin;
impl Plugin for StardustSharedPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, network_pre_update);
        app.add_systems(PostUpdate, network_post_update);

        app.add_systems(PreUpdate, panic_on_channel_removal);

        app.insert_resource(UniqueNetworkHasher(Box::new(DefaultHasher::default())));
    }

    fn finish(&self, app: &mut App) {
        if app.is_plugin_added::<StardustServerPlugin>() && app.is_plugin_added::<StardustClientPlugin>() {
            panic!("You can't be both a client and a server!");
        }
        
        // Finalize hash
        let hasher = app.world.remove_resource::<UniqueNetworkHasher>().unwrap();
        let int = hasher.0.finish();
        let hex = format!("{:X}", int);
        app.world.insert_resource(UniqueNetworkHash { int, hex });
    }
}