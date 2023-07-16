use bevy::prelude::Resource;

#[derive(Resource, Debug, Clone)]
pub struct ServerConfig {
    pub max_players: u32,
}