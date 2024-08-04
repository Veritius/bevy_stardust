use std::{net::{SocketAddr, UdpSocket}, sync::Arc};
use anyhow::Result;
use bevy::{ecs::{entity::Entities, system::SystemParam}, prelude::*};
use quinn_proto::{ClientConfig, EndpointConfig, ServerConfig};

/// Utility for opening endpoints.
#[derive(SystemParam)]
pub struct Manager<'w> {
    entities: &'w Entities,
}

impl Manager<'_> {
    /// Queues a new endpoint to be opened.
    pub fn open_endpoint(
        &mut self,
        endpoint_config: Arc<EndpointConfig>,
        server_config: Option<Arc<ServerConfig>>,
        bind_address: SocketAddr,
    ) -> Result<Entity> {
        let socket = UdpSocket::bind(bind_address)?;
        socket.set_nonblocking(true)?;

        let quinn = quinn_proto::Endpoint::new(
            endpoint_config,
            server_config,
            true,
            None,
        );

        todo!()
    }

    /// Queues a new connection to be opened.
    /// 
    /// Note that even if this returns `Ok`, the connection may fail to open.
    pub fn open_connection(
        &mut self,
        endpoint: Entity,
        client_config: ClientConfig,
        remote_address: SocketAddr,
        server_name: Arc<str>,
    ) -> Result<Entity> {
        todo!()
    }
}

pub(crate) struct QueuedEndpointOpen {
    pub entity: Entity,
    pub endpoint: Box<quinn_proto::Endpoint>,

    pub connections: Vec<ConnectionConfig>,
}

pub(crate) struct QueuedConnectionOpen {
    pub entity: Entity,
    pub endpoint: Entity,

    pub config: ConnectionConfig,
}

pub(crate) struct ConnectionConfig {
    pub client_config: ClientConfig,
    pub remote_address: SocketAddr,
    pub server_name: Arc<str>,
}