use bevy::prelude::*;
use bevy_stardust::prelude::*;
use crate::{Endpoint, Connection};

/// Adds QUIC support to the `App`.
pub struct QuicPlugin;

impl Plugin for QuicPlugin {
    fn name(&self) -> &str { "QuicPlugin" }

    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<StardustPlugin>() {
            panic!("StardustPlugin must be added before QuicPlugin");
        }

        app.register_type::<Endpoint>();
        app.register_type::<Connection>();

        #[cfg(feature="quiche")]
        crate::quiche::setup(app);
    }
}