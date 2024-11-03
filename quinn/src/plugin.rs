use bevy_app::prelude::*;
use crate::config::BackendConfig;

/// Adds QUIC support to the application.
pub struct QuicPlugin {
    /// Configuration for the async backend.
    pub backend: BackendConfig,
}

impl Plugin for QuicPlugin {
    fn build(&self, app: &mut App) {
        todo!()
    }
}