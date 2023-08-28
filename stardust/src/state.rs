use bevy::{prelude::*, reflect::Reflect};

/// The current mode of the game.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Reflect, States)]
pub enum NetworkMode {
    /// No network functionality.
    #[default]
    Offline,
    /// Running as a server.
    Server,
    /// Running as a client.
    Client,
}