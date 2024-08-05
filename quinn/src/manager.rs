use std::{net::{SocketAddr, ToSocketAddrs}, sync::Arc};
use anyhow::Result;
use bevy::{ecs::{query::QueryEntityError, system::{EntityCommands, SystemParam}}, prelude::*};
use quinn_proto::{ClientConfig, EndpointConfig, ServerConfig};

/// Utility for opening endpoints.
#[derive(SystemParam)]
pub struct Endpoints<'w, 's> {
    commands: Commands<'w, 's>,
}

impl Endpoints<'_, '_> {
    /// Queues a new endpoint to be opened.
    pub fn create(
        &mut self,
        endpoint_config: Arc<EndpointConfig>,
        server_config: Option<Arc<ServerConfig>>,
        bind_address: impl ToSocketAddrs,
    ) -> EndpointCommands {
        todo!()
    }

    /// Gets an [`EndpointCommands`] for an existing endpoint.
    pub fn endpoint(
        &mut self,
        endpoint: Entity,
    ) -> Result<EndpointCommands, QueryEntityError> {
        todo!()
    }
}

pub struct EndpointCommands<'a> {
    commands: EntityCommands<'a>,
}

impl<'a> EndpointCommands<'a> {
    pub fn id(&self) -> Entity {
        self.commands.id()
    }

    pub fn insert(
        &mut self,
        components: impl Bundle,
    ) -> EndpointCommands<'a> {
        todo!()
    }

    pub fn connect(
        &mut self,
        client_config: ClientConfig,
        remote_address: SocketAddr,
        server_name: Arc<str>,
    ) -> ConnectionCommands<'a> {
        todo!()
    }
}

pub struct ConnectionCommands<'a> {
    commands: EntityCommands<'a>,
}

impl<'a> ConnectionCommands<'a> {
    pub fn id(&self) -> Entity {
        self.commands.id()
    }

    pub fn insert(
        &mut self,
        components: impl Bundle,
    ) -> ConnectionCommands<'a> {
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