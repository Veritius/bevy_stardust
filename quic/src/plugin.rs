use bevy::prelude::*;
use bevy_stardust::prelude::*;
use crate::{Endpoint, Connection};

/// Adds QUIC support to the `App`.
pub struct QuicPlugin;

impl Plugin for QuicPlugin {
    fn name(&self) -> &str { "QuicPlugin" }

    fn build(&self, app: &mut App) {
        app.configure_sets(PreUpdate, QuicSystems::ReceivePackets.in_set(NetworkRecv::Receive));
        app.configure_sets(PostUpdate, QuicSystems::ReceivePackets.in_set(NetworkSend::Transmit));

        app.register_type::<Endpoint>();
        app.register_type::<Connection>();

        app.add_event::<crate::events::TryConnectEvent>();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub(crate) enum QuicSystems {
    ReceivePackets,
    TransmitPackets,
}