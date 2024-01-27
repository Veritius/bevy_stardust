use bevy::prelude::*;

#[derive(Resource)]
pub(crate) struct PluginConfig {
    pub river_count: u16,
    pub bitfield_bytes: u8,
}

#[derive(Resource)]
pub(crate) struct ConnectionConfig {
    pub allow_incoming_connections: bool,
    pub maximum_active_connections: u32,
}