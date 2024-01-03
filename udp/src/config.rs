use bevy::prelude::*;

#[derive(Resource)]
pub(crate) struct PluginConfig {
    pub reliability_bitfield_bytes: u8,
}