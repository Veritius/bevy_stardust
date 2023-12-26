use std::{net::SocketAddr, sync::Arc};
use bevy::prelude::*;
use quinn_proto::{ServerConfig, ClientConfig};

#[derive(Event)]
pub(crate) enum EndpointManagerEvent {
    StartServer {
        address: SocketAddr,
        capacity: u32,
        config: Arc<ServerConfig>,
    },
    StartClient {
        address: SocketAddr,
    },
    CloseEndpoint,
    TryConnect {
        address: SocketAddr,
        config: ClientConfig,
    },
    SetIncoming {
        allowed: bool,
    }
}