use std::{net::SocketAddr, sync::Arc};
use bevy::{ecs::world::Command, prelude::*};
use quinn_proto::{EndpointConfig, ServerConfig};
use crate::{endpoints::{BuildingEndpoint, EndpointMetadata}, Endpoint};

/// A command to open a new [`Endpoint`](crate::Endpoint).
pub struct OpenEndpoint {
    building: Box<BuildingEndpoint>,
}

impl OpenEndpoint {
    /// Creates an [`OpenEndpoint`] command, used to create new [`Endpoint`] instances.
    /// 
    /// If successful, returns the local address the endpoint is bound to.
    /// 
    /// ## Servers
    /// If `server_config` is `None`, the endpoint will not be able to act as a server.
    /// The server config can be added or replaced at any time by using [`set_server_config`](Endpoint::set_server_config).
    /// 
    /// Endpoints will always be able to act as a client.
    /// 
    /// ## Binding
    /// If `bind_address` is `None`, the OS will automatically assign an address to the socket.
    /// This is useful for clients, which don't need to have a known IP/port, but can make servers
    /// unreachable, such as in cases where port forwarding is needed.
    /// 
    /// If there is already a socket at the given address, `Err` is returned.
    pub fn new(
        bind_address: Option<SocketAddr>,
        endpoint_config: Arc<EndpointConfig>,
        server_config: Option<Arc<ServerConfig>>,
    ) -> anyhow::Result<Self> {
        let building = Endpoint::new_inner(
            bind_address,
            endpoint_config,
            server_config
        )?;

        // We're done for now
        return Ok(Self { building });
    }
}

impl Command for OpenEndpoint {
    fn apply(self, world: &mut World) {
        #[cfg(debug_assertions)]
        let world_id = world.id();

        // Spawn a new entity for our endpoint
        let mut entity = world.spawn_empty();

        // Endpoint metadata
        let meta = EndpointMetadata {
            eid: entity.id(),

            #[cfg(debug_assertions)]
            world: world_id,
        };

        // Done: building is complete
        entity.insert(self.building.finish(meta));
    }
}