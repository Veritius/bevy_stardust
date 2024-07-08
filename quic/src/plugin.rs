use bevy::prelude::*;
use bevy_stardust::prelude::*;
use crate::{Endpoint, Connection};

/// Adds QUIC support to the `App`.
pub struct QuicPlugin;

impl Plugin for QuicPlugin {
    fn name(&self) -> &str {
        "QuicPlugin"
    }

    fn build(&self, app: &mut App) {
        app.register_type::<Endpoint>();
        app.register_type::<Connection>();

        app.add_systems(PreUpdate, (
            crate::receiving::endpoints_receive_datagrams_system,
        ).chain().in_set(NetworkRecv::Receive));
    }
}