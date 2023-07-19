use bevy::prelude::*;
use bevy_stardust_shared::{plugin::StardustSharedPlugin};
use crate::{connection::NetworkSocket, config::ServerConfig, auth::{config::AuthenticationServerConfig, server::start_auth_server}};

pub struct StardustServerPlugin {
    pub config: ServerConfig,
    pub bind_port: u16,
}

impl Plugin for StardustServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(StardustSharedPlugin);
        app.insert_resource(NetworkSocket::new(self.bind_port));
        app.insert_resource(self.config.clone());
    }

    fn cleanup(&self, app: &mut App) {
        let cfg = app.world
            .remove_resource::<AuthenticationServerConfig>()
            .expect("Authentication server was never configured!");

        let auth_server = start_auth_server(cfg.address, cfg.certificates, cfg.private_key);
    }
}