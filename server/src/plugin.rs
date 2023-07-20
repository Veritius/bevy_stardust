use bevy::prelude::*;
use crate::{connection::NetworkSocket, config::ServerConfig};

pub struct StardustServerPlugin {
    pub config: ServerConfig,
    /// The port for the UDP socket to bind to.
    pub bind_port: u16,
}

impl Plugin for StardustServerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(NetworkSocket::new(self.bind_port));
        app.insert_resource(self.config.clone());
    }
}