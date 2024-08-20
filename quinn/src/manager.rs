use std::{net::ToSocketAddrs, sync::Arc};
use anyhow::Result;
use bevy::{ecs::{entity::Entities, system::{EntityCommands, SystemParam}}, prelude::*};
use quinn_proto::{ClientConfig, EndpointConfig, ServerConfig};

/// Utility for opening endpoints.
#[derive(SystemParam)]
pub struct Endpoints<'w, 's> {
    commands: Commands<'w, 's>,
    entities: &'w Entities,
}

impl Endpoints<'_, '_> {
    /// Queues a new endpoint to be opened.
    pub fn open(
        &mut self,
        f: impl FnOnce(&mut EndpointBuilder),
    ) {
        let mut builder = EndpointBuilder {
            commands: self.commands.reborrow(),
        };

        f(&mut builder);
    }
}

pub struct EndpointBuilder<'a> {
    commands: Commands<'a, 'a>,
}

impl<'a> EndpointBuilder<'a> {
    pub fn simple(
        &mut self,
        endpoint_config: Arc<EndpointConfig>,
        server_config: Option<Arc<ServerConfig>>,
        bind_address: impl ToSocketAddrs,
    ) -> Result<EndpointCommands<'a>> {
        todo!()
    }
}

pub struct EndpointCommands<'a> {
    commands: EntityCommands<'a>,
}

impl<'a> EndpointCommands<'a> {
    pub fn connect(
        &mut self,
        f: impl FnOnce(&mut ConnectionBuilder),
    ) {
        let mut builder = ConnectionBuilder {
            endpoint: self.commands.id(),
            commands: self.commands.commands(),
        };

        f(&mut builder);
    }
}

pub struct ConnectionBuilder<'a> {
    endpoint: Entity,
    commands: Commands<'a, 'a>,
}

impl<'a> ConnectionBuilder<'a> {
    pub fn simple(
        &mut self,
        client_config: ClientConfig,
        remote_address: impl ToSocketAddrs,
        server_name: Arc<str>,
    ) -> Result<ConnectionCommands> {
        todo!()
    }
}

pub struct ConnectionCommands<'a> {
    commands: EntityCommands<'a>,
}