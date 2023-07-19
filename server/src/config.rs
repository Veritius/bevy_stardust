use bevy::prelude::Resource;

/// Various options about how Stardust should manage the server.
/// You can change this at any time, even when the server is running.
#[derive(Resource, Debug, Clone)]
pub struct ServerConfig {
    /// The soft max amount of players that can be connected at any one time.
    pub max_players: u16,
}