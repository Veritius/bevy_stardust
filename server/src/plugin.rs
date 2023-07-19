use bevy::prelude::*;
use crate::{connection::NetworkSocket, config::ServerConfig, auth::{config::AuthenticationServerConfig, server::start_auth_server}};

pub struct StardustServerPlugin {
    pub config: ServerConfig,
    /// The port for the TLS webserver to bind to. This should be different to `udp_bind_port`.
    pub tls_bind_port: u16,
    /// The port for the UDP socket to bind to. This should be different to `tls_bind_port`.
    pub udp_bind_port: u16,
}

impl Plugin for StardustServerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(NetworkSocket::new(self.udp_bind_port));
        app.insert_resource(self.config.clone());
    }

    fn cleanup(&self, app: &mut App) {
        let cfg = app.world
            .remove_resource::<AuthenticationServerConfig>()
            .expect("Authentication server was never configured!");

        let auth_server = start_auth_server(cfg.address, cfg.certificates, cfg.private_key);
        app.insert_resource(auth_server);
    }
}