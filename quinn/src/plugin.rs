use bevy_ecs::prelude::*;
use bevy_app::prelude::*;
use bevy_stardust::prelude::*;

/// The core plugin for `bevy_stardust_quinn`.
pub struct QuinnPlugin;

impl Plugin for QuinnPlugin {
    fn build(&self, app: &mut App) {
        // default crypto provider for tls
        install_default_crypto_provider();

        // PreUpdate stage
        app.add_systems(PreUpdate, (
            crate::endpoint::io_udp_recv_system,
            crate::systems::event_exchange_system,
            crate::systems::event_polling_system,
            crate::systems::poll_incoming_messages_system,
        ).chain().in_set(NetworkRecv::Receive));

        // PostUpdate stage
        app.add_systems(PostUpdate, (
            crate::systems::put_outgoing_messages_system,
            crate::systems::application_exit_system,
            crate::systems::event_exchange_system,
            crate::systems::event_polling_system,
            crate::endpoint::io_udp_send_system,
        ).chain().in_set(NetworkSend::Transmit));
    }
}

fn install_default_crypto_provider() -> bool {
    rustls::crypto::ring::default_provider()
        .install_default()
        .is_ok()
}