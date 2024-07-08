use bevy::prelude::*;

/// Adds QUIC support to the `App`.
pub struct QuicPlugin;

impl Plugin for QuicPlugin {
    fn name(&self) -> &str {
        "QuicPlugin"
    }

    fn build(&self, app: &mut App) {
        todo!()
    }
}