use bevy::prelude::*;
use crate::{Endpoint, Connection};

/// Adds QUIC support to the `App`.
pub struct QuicPlugin;

impl Plugin for QuicPlugin {
    fn name(&self) -> &str {
        "QuicPlugin"
    }

    fn build(&self, app: &mut App) {
        app.register_type::<Endpoint>();
        app.register_type::<Connection>();
    }
}