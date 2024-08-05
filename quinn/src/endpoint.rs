use std::{net::{SocketAddr, UdpSocket}, sync::Arc};
use anyhow::Result;
use bevy::prelude::*;
use quinn::{ClientConfig, Endpoint, EndpointConfig, ServerConfig, VarInt};
use crate::QuinnConnection;

/// Represents one Quinn endpoint.
#[derive(Component)]
pub struct QuinnEndpoint {
    endpoint: Endpoint,
}

impl QuinnEndpoint {
    /// Create a new endpoint attached to `udp_socket`.
    pub fn new(
        endpoint_config: EndpointConfig,
        server_config: Option<ServerConfig>,
        udp_socket: UdpSocket,
    ) -> Result<Self> {
        Ok(Self {
            endpoint: Endpoint::new(
                endpoint_config,
                server_config,
                udp_socket,
                Arc::new(quinn::TokioRuntime),
            )?,
        })
    }

    /// Opens a connection to `address`.
    pub fn connect(
        &self,
        config: ClientConfig,
        address: SocketAddr,
        server_name: &str,
    ) -> Result<QuinnConnection> {
        let connecting = self.endpoint.connect_with(
            config,
            address,
            server_name
        )?;

        return Ok(QuinnConnection::connecting(connecting));
    }

    /// Closes all connections immediately and stops accepting new connections.
    pub fn close(&self, code: u32, reason: &[u8]) {
        self.endpoint.close(VarInt::from_u32(code), reason);
    }
}