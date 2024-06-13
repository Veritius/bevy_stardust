use bevy::prelude::*;
use bevy_stardust::prelude::*;
use crate::connections::*;
use crate::endpoints::*;

/// Adds QUIC functionality to the application.
pub struct QuicTransportPlugin;

impl Plugin for QuicTransportPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, endpoint_datagram_recv_system
            .in_set(NetworkRead::Receive));

        app.add_systems(PreUpdate, connection_event_handler_system
            .in_set(NetworkRead::Receive)
            .after(endpoint_datagram_recv_system));

        app.add_systems(PostUpdate, connection_message_sender_system
            .in_set(NetworkWrite::Send));

        app.add_systems(PostUpdate, connection_datagram_send_system
            .in_set(NetworkWrite::Send)
            .after(connection_message_sender_system));
    }
}