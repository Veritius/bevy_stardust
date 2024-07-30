use std::net::UdpSocket;
use anyhow::Result;
use bevy::prelude::*;
use quinn::{EndpointConfig, ServerConfig};

/// Represents one Quinn endpoint.
#[derive(Component)]
pub struct QuinnEndpoint {

}

impl QuinnEndpoint {
    /// Create a new endpoint attached to `udp_socket`.
    pub fn new(
        endpoint_config: EndpointConfig,
        server_config: Option<ServerConfig>,
        udp_socket: UdpSocket,
    ) -> Result<Self> {
        todo!()
    }
}