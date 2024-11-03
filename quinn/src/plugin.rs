use bevy_app::prelude::*;
use crate::{backend::executor::BackendExecutor, config::BackendConfig};

/// Adds QUIC support to the application.
pub struct QuicPlugin {
    /// Configuration for the async backend.
    pub backend: BackendConfig,
}

impl Plugin for QuicPlugin {
    fn build(&self, app: &mut App) {
        // Add the backend executor
        app.insert_resource(BackendExecutor::init(&self.backend));
    }
}