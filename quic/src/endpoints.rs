use std::{net::SocketAddr, sync::Arc, time::Instant};
use anyhow::Result;
use bevy::{ecs::entity::EntityHashMap, prelude::*};
use quinn_proto::{ClientConfig, ConnectionHandle, Endpoint, EndpointConfig, ServerConfig};

use crate::QuicConnection;

/// A QUIC endpoint.
/// 
/// # Safety
/// This component must always stay in the same [`World`] as it was created in.
/// Being put into another `World` will lead to undefined behavior.
#[derive(Component)]
pub struct QuicEndpoint {
    pub(crate) inner: Box<Endpoint>,
    pub(crate) handles: EntityHashMap<ConnectionHandle>,
}

impl QuicEndpoint {
    /// Creates a new endpoint.
    /// 
    /// If `server_config` is `None`, incoming connections will be rejected.
    /// This can be changed at any time with [`set_server_config`](Self::set_server_config).
    pub fn new(
        config: Arc<EndpointConfig>,
        server_config: Option<Arc<ServerConfig>>,
        allow_mtud: bool,
        rng_seed: Option<[u8; 32]>,
    ) -> Result<Self> {
        Ok(Self {
            inner: Box::new(Endpoint::new(config, server_config, allow_mtud, rng_seed)),
            handles: EntityHashMap::default(),
        })
    }

    /// Replaces the server config of the endpoint, affecting new connections only.
    /// If `None`, the endpoint cannot act as a server, and incoming connections will be rejected.
    pub fn set_server_config(
        &mut self,
        server_config: Option<Arc<ServerConfig>>,
    ) {
        self.inner.set_server_config(server_config)
    }

    /// Connects to a new connection.
    /// 
    /// # Safety
    /// `commands` must only be applied to the world the `QuicEndpoint` is part of.
    pub fn connect(
        &mut self,
        commands: &mut Commands,
        config: ClientConfig,
        remote: SocketAddr,
        server_name: &str,
    ) -> Result<Entity> {
        // Create the connection data
        let (handle, connection) = self.inner.connect(Instant::now(), config, remote, server_name)?;

        // Spawn the connection entity
        let entity = commands.spawn(QuicConnection {
            handle,
            inner: Box::new(connection),
        }).id();

        // Return the entity id
        return Ok(entity);
    }
}