use std::{net::SocketAddr, sync::Arc};
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
        todo!()
    }

    /// Queues a new connection to be opened.
    /// 
    /// Note that even if this returns `Ok`, the connection may fail to open.
    pub fn open_connection(
        &mut self,
        endpoint: Entity,
        client_config: ClientConfig,
        remote: SocketAddr,
        server_name: &str,
    ) -> Result<Entity> {
        todo!()
    }
}