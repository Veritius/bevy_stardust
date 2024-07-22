use bevy::prelude::*;
use bevy_stardust::prelude::*;

/// Adds QUIC support to the `App`.
pub struct QuicPlugin;

impl Plugin for QuicPlugin {
    fn name(&self) -> &str { "QuicPlugin" }

    fn build(&self, app: &mut App) {
        app.configure_sets(PreUpdate, QuicSystems::ReceivePackets.in_set(NetworkRecv::Receive));
        app.configure_sets(PostUpdate, QuicSystems::ReceivePackets.in_set(NetworkSend::Transmit));

        app.add_event::<crate::events::TryConnectEvent>();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub(crate) enum QuicSystems {
    ReceivePackets,
    TransmitPackets,
}