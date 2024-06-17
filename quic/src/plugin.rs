use bevy::prelude::*;
use bevy_stardust::prelude::*;
use crate::connections::*;
use crate::endpoints::*;

/// Adds QUIC functionality to the application.
pub struct QuicPlugin;

impl Plugin for QuicPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, (
            endpoint_datagram_recv_system,
            connection_endpoint_events_system,
            connection_event_handler_system,
        ).chain().in_set(NetworkRead::Receive));

        app.add_systems(PostUpdate, (
            connection_message_sender_system,
            connection_datagram_send_system,
            connection_endpoint_events_system,
        ).chain().in_set(NetworkWrite::Send));
    }
}