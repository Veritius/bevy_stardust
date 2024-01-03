use bevy::prelude::*;

#[derive(Resource)]
pub(crate) struct PluginConfig {
    pub bitfield_bytes: u8,
}