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
            crate::endpoints::udp_recv_system,
            crate::endpoints::event_exchange_system,
            crate::connections::connection_events_system,
            crate::connections::qsm_events_system,
        ).chain().in_set(NetworkRecv::Receive));

        // PostUpdate stage
        app.add_systems(PostUpdate, (
            crate::connections::outgoing_messages_system,
            crate::connections::qsm_events_system,
            crate::endpoints::event_exchange_system,
            crate::endpoints::udp_send_system,
        ).chain().in_set(NetworkSend::Transmit));
    }
}

fn install_default_crypto_provider() -> bool {
    rustls::crypto::ring::default_provider()
        .install_default()
        .is_ok()
}