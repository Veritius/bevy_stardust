use std::{net::{ToSocketAddrs, UdpSocket}, sync::Arc};
use anyhow::Result;
use bevy::{ecs::{entity::Entities, system::{EntityCommands, SystemParam}}, prelude::*};
use quinn_proto::{ClientConfig, EndpointConfig, ServerConfig};

use crate::Endpoint;

/// Utility for opening endpoints.
#[derive(SystemParam)]
pub struct Endpoints<'w, 's> {
    commands: Commands<'w, 's>,
    entities: &'w Entities,
}

impl Endpoints<'_, '_> {
    /// Opens a new endpoint.
    /// 
    /// Returns an error immediately if the configuration is invalid.
    pub fn open(
        &mut self,
        endpoint_config: Arc<EndpointConfig>,
        server_config: Option<Arc<ServerConfig>>,
        bind_address: impl ToSocketAddrs,
        f: impl FnOnce(&mut EndpointCommands),
    ) -> Result<()> {
        // Bind and configure the UDP socket for communication
        let socket = UdpSocket::bind(bind_address)?;
        socket.set_nonblocking(true)?;

        // Create the Quinn endpoint we will be using
        let endpoint = quinn_proto::Endpoint::new(
            endpoint_config,
            server_config,
            true,
            None
        );

        // Create the endpoint component
        let mut endpoint = Endpoint::new(
            socket,
            endpoint
        );

        // Spawn the new endpoint entity
        let commands = self.commands.spawn_empty();

        // Create the commands object
        let mut ep_cmds = EndpointCommands {
            endpoint: &mut endpoint,
            commands: todo!(),
        };

        // Run the user commands thingy
        f(&mut ep_cmds);

        // Insert the endpoint component into the entity
        commands.insert(endpoint);

        return Ok(());
    }
}

pub struct EndpointCommands<'a> {
    endpoint: &'a mut Endpoint,
    commands: EntityCommands<'a>,
}

impl<'a> EndpointCommands<'a> {
    /// Attempts to connect through an endpoint.
    /// 
    /// Returns an error immediately if the configuration is invalud.
    /// This may still fail later after creation.
    pub fn connect(
        &mut self,
        client_config: ClientConfig,
        remote_address: impl ToSocketAddrs,
        server_name: &str,
        f: impl FnOnce(&mut ConnectionCommands),
    ) -> Result<()> {
        todo!();

        let mut cn_cmds = ConnectionCommands {
            commands: todo!(),
        };

        f(&mut cn_cmds);
    }
}

pub struct ConnectionCommands<'a> {
    commands: EntityCommands<'a>,
}