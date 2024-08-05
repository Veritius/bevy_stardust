use std::{marker::PhantomData, net::{SocketAddr, ToSocketAddrs}, sync::Arc};
use bevy::{ecs::{entity::Entities, system::{EntityCommand, SystemParam}}, prelude::*};
use quinn_proto::{ClientConfig, EndpointConfig, ServerConfig};

/// Utility for opening endpoints.
#[derive(SystemParam)]
pub struct Endpoints<'w, 's> {
    commands: Commands<'w, 's>,
    entities: &'w Entities,
}

impl Endpoints<'_, '_> {
    /// Queues a new endpoint to be opened.
    pub fn create(
        &mut self,
        endpoint_config: Arc<EndpointConfig>,
        server_config: Option<Arc<ServerConfig>>,
        bind_address: impl ToSocketAddrs,
    ) -> EndpointBuilder {
        let id = self.entities.reserve_entity();

        return EndpointBuilder {
            entities: self.entities,

            endpoint: QueuedEndpoint {
                entity: id,

                endpoint_config,
                server_config,
                bind_address: todo!(),

                connections: Vec::new(),
                commands: Vec::new(),
            },
        };
    }
}

pub struct EndpointBuilder<'a> {
    entities: &'a Entities,

    endpoint: QueuedEndpoint,
}

impl<'a> EndpointBuilder<'a> {
    pub fn id(&self) -> Entity {
        self.endpoint.entity
    }

    pub fn add(
        &mut self,
        command: impl EntityCommand,
    ) -> &mut EndpointBuilder<'a> {
        self.endpoint.commands.push(Box::new(command));
        return self;
    }

    pub fn connect(
        &mut self,
        client_config: ClientConfig,
        remote_address: SocketAddr,
        server_name: Arc<str>,
    ) -> ConnectionBuilder<'a> {
        let id = self.entities.reserve_entity();

        return ConnectionBuilder {
            connection: QueuedConnection {
                entity: id,
                client_config,
                remote_address,
                server_name,
                commands: Vec::new(),
            },

            _ph: PhantomData,
        };
    }
}

struct QueuedEndpoint {
    entity: Entity,

    endpoint_config: Arc<EndpointConfig>,
    server_config: Option<Arc<ServerConfig>>,
    bind_address: SocketAddr,

    connections: Vec<QueuedConnection>,
    commands: Vec<Box<dyn EntityCommand>>,
}

pub struct ConnectionBuilder<'a> {
    connection: QueuedConnection,
    _ph: PhantomData<&'a ()>,
}

impl<'a> ConnectionBuilder<'a> {
    pub fn id(&self) -> Entity {
        self.connection.entity
    }

    pub fn add(
        &mut self,
        command: impl EntityCommand,
    ) -> &mut ConnectionBuilder<'a> {
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