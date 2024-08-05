use bevy::prelude::*;
use bevy_stardust::prelude::*;

/// Adds QUIC transport support based on `quiche`.
pub struct QuichePlugin;

impl Plugin for QuichePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, (
            crate::endpoint::endpoint_recv_packets_system,
            crate::connection::connection_event_handling_system,
            (
                crate::endpoint::endpoint_handle_events_system,
                crate::connection::connection_message_recv_system,
            ),
        ).chain().in_set(NetworkRecv::Receive));

        app.add_systems(PostUpdate, (
            crate::connection::connection_message_send_system,
            crate::connection::connection_event_handling_system,
            crate::endpoint::endpoint_handle_events_system,
        ).chain().in_set(NetworkSend::Transmit));
    }
}