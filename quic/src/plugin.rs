use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use crate::EndpointHandler;

/// Adds QUIC support.
pub struct QuicPlugin;

impl Plugin for QuicPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, EndpointHandler::system
            .run_if(resource_exists::<EndpointHandler>));
    }
}