use bevy::prelude::*;
use bevy_stardust::prelude::*;

/// Adds QUIC support using Quinn.
pub struct QuinnPlugin;

impl Plugin for QuinnPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, crate::connection::message_recv_system
            .in_set(NetworkRecv::Receive));

        app.add_systems(PostUpdate, crate::connection::message_send_system
            .in_set(NetworkSend::Transmit));
    }
}