use bevy::prelude::*;

#[derive(Resource)]
pub(crate) struct PluginConfig {
    pub river_count: u16,
    pub bitfield_bytes: u8,
}