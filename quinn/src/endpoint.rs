use std::{net::UdpSocket, sync::Arc};
use anyhow::Result;
use bevy::prelude::*;
use quinn::{Endpoint, EndpointConfig, ServerConfig, TokioRuntime};

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
                Arc::new(TokioRuntime),
            )?,
        })
    }
}