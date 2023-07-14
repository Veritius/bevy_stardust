use bevy::prelude::*;

pub struct StardustServerPlugin {
    pub private_key: String,
    pub max_players: u32,
    pub bind_port: u16,
}

impl Plugin for StardustServerPlugin {
    fn build(&self, app: &mut App) {
        todo!()
    }
}