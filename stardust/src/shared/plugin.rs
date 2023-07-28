use bevy::prelude::*;
use crate::{shared::protocol::ProtocolBuilder, server::plugin::StardustServerPlugin, client::plugin::StardustClientPlugin};
use super::{scheduling::{network_pre_update, network_post_update}, channels::systems::panic_on_channel_removal};

/// Shared information between the client and server.
/// See the demos for information on how to use this.
pub struct StardustSharedPlugin {}
impl Plugin for StardustSharedPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ProtocolBuilder::default());
        app.add_systems(PreUpdate, network_pre_update);
        app.add_systems(PostUpdate, network_post_update);

        app.add_systems(PreUpdate, panic_on_channel_removal);
    }

    fn finish(&self, app: &mut App) {
        if app.is_plugin_added::<StardustServerPlugin>() && app.is_plugin_added::<StardustClientPlugin>() {
            panic!("You can't be both a client and a server!");
        }

        let protocol = app.world.remove_resource::<ProtocolBuilder>()
            .expect("Builder should have been present").build();

        info!("Protocol ID set to {}", protocol.id());
        app.world.insert_resource(protocol);
    }
}