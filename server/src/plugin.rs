use bevy::prelude::*;
use bevy_stardust_shared::plugin::StardustSharedPlugin;

pub struct StardustServerPlugin {
    pub private_key: String,
    pub max_players: u32,
    pub bind_port: u16,
}

impl Plugin for StardustServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(StardustSharedPlugin);
    }
}