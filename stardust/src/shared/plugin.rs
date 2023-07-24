use bevy::prelude::*;
use crate::shared::protocol::ProtocolBuilder;
use super::scheduling::{network_pre_update, network_post_update};

/// Shared information between the client and server.
/// See the demos for information on how to use this.
pub struct StardustSharedPlugin {}
impl Plugin for StardustSharedPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ProtocolBuilder::default());
        app.add_systems(PreUpdate, network_pre_update);
        app.add_systems(PostUpdate, network_post_update);
    }

    fn finish(&self, app: &mut App) {
        let protocol = app.world.remove_resource::<ProtocolBuilder>()
            .expect("Builder should have been present").build();

        info!("Protocol ID set to {}", protocol.id());
        app.world.insert_resource(protocol);
    }
}