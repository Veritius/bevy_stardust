use std::{marker::PhantomData, net::{SocketAddr, ToSocketAddrs}, sync::Arc};
use anyhow::Result;
use bevy::{ecs::{entity::Entities, query::QueryEntityError, system::{EntityCommand, EntityCommands, SystemParam}, world::CommandQueue}, prelude::*};
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
    connection: QueuedEndpoint,
    entities: &'a Entities,
}

impl<'a> EndpointCommands<'a> {
    pub fn id(&self) -> Entity {
        self.connection.entity
    }

    pub fn add(
        &mut self,
        command: impl EntityCommand,
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

struct QueuedEndpoint {
    entity: Entity,

    endpoint: Box<quinn_proto::Endpoint>,

    connections: Vec<Box<QueuedConnection>>,

    commands: Vec<Box<dyn EntityCommand>>,
}

pub struct ConnectionCommands<'a> {
    connection: QueuedConnection,
    _ph: PhantomData<&'a ()>,
}

impl<'a> ConnectionCommands<'a> {
    pub fn id(&self) -> Entity {
        self.connection.entity
    }

    pub fn add(
        &mut self,
        command: impl EntityCommand,
    ) -> &mut ConnectionCommands<'a> {
        self.connection.commands.push(Box::new(command));
        return self;
    }
}

struct QueuedConnection {
    entity: Entity,

    client_config: ClientConfig,
    remote_address: SocketAddr,
    server_name: Arc<str>,

    commands: Vec<Box<dyn EntityCommand>>,
}