use std::{net::SocketAddr, sync::Arc};
use bevy_ecs::{prelude::*, system::EntityCommand};
use quinn_proto::{ClientConfig, EndpointConfig};
use rustls::ServerConfig;

/// Creates a new QUIC endpoint with this entity.
pub struct MakeEndpoint {
    /// The local address to bind a UDP socket to.
    pub address: SocketAddr,

    /// The configuration of the endpoint.
    pub config: Arc<EndpointConfig>,

    /// The server configuration of the endpoint.
    pub server: Option<Arc<ServerConfig>>,
}

impl EntityCommand for MakeEndpoint {
    fn apply(
        self,
        id: Entity,
        world: &mut World,
    ) {
        todo!()
    }
}

/// Creates a new QUIC connection based on an endpoint.
pub struct OpenConnection {
    /// The address of the remote server to connect to.
    pub remote: SocketAddr,

    /// The configuration of the client.
    pub config: ClientConfig,
}

impl EntityCommand for OpenConnection {
    fn apply(
        self,
        id: Entity,
        world: &mut World,
    ) {
        todo!()
    }
}