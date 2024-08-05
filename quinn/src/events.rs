use std::{net::SocketAddr, sync::Arc};
use bevy::prelude::*;
use quinn_proto::{ClientConfig, EndpointConfig, ServerConfig};

/// Sent by systems to open a new endpoint.
#[derive(Event)]
pub struct OpenEndpointEvent {
    pub endpoint_config: Arc<EndpointConfig>,
    pub server_config: Option<Arc<ServerConfig>>,
    pub bind_address: SocketAddr,

    pub connections: Vec<NewConnection>,
}

pub struct NewConnection {
    pub client_config: ClientConfig,
    pub remote_address: SocketAddr,
    pub server_name: Arc<str>,
}