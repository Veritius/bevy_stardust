use bevy::prelude::*;
use bevy_stardust::prelude::*;
use crate::backend::QuicBackend;

/// Adds QUIC support to the `App`.
pub struct QuicPlugin;

impl Plugin for QuicPlugin {
    fn name(&self) -> &str { "QuicPlugin" }

    fn build(&self, app: &mut App) {
        app.configure_sets(PreUpdate, QuicSystems::ReceivePackets.in_set(NetworkRecv::Receive));
        app.configure_sets(PostUpdate, QuicSystems::ReceivePackets.in_set(NetworkSend::Transmit));

        app.add_event::<crate::events::TryConnectEvent>();

        #[cfg(feature="quiche")]
        crate::quiche::setup(app);
    }
}

/// Adds a backend for QUIC plugins.
pub struct QuicBackendPlugin<B: QuicBackend>(B);

impl<B: QuicBackend> QuicBackendPlugin<B> {
    /// Creates a new backend plugin using `backend`.
    pub fn new(backend: B) -> QuicBackendPlugin<B> {
        Self(backend)
    }
}

impl<B: QuicBackend> Plugin for QuicBackendPlugin<B> {
    fn build(&self, app: &mut App) {
        todo!()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub enum QuicSystems {
    ReceivePackets,
    TransmitPackets,
}