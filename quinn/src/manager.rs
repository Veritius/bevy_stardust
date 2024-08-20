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
        todo!();

        let mut ep_cmds = EndpointCommands {
            commands: todo!(),
        };

        f(&mut ep_cmds);

        return Ok(());
    }
}

pub struct EndpointCommands<'a> {
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
        server_name: Arc<str>,
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