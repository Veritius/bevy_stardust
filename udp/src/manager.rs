use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, ToSocketAddrs};
use anyhow::{Context, Result};
use bevy::{prelude::*, ecs::{entity::Entities, system::SystemParam}};
use crate::endpoint::{ConnectionOwnershipToken, Endpoint};

/// A SystemParam that lets you create [`Endpoints`](Endpoint) and open outgoing [`Connections`](Connection).
#[derive(SystemParam)]
pub struct UdpManager<'w, 's> {
    entities: &'w Entities,
    commands: Commands<'w, 's>,
    endpoints: Query<'w, 's, &'static mut Endpoint>,
}

impl UdpManager<'_, '_> {
    /// Opens an endpoint bound to the address returned by [`to_socket_addrs`](ToSocketAddrs::to_socket_addrs).
    pub fn open_endpoint(
        &mut self,
        address: impl ToSocketAddrs,
        listener: bool,
    ) -> Result<Entity> {
        // Resolve address
        let address = resolve_address(address)?;

        // Reserve endpoint id
        let endpoint_id = self.entities.reserve_entity();

        // Create endpoint
        let mut endpoint = self.open_endpoint_inner(endpoint_id, address)?;
        endpoint.listening = listener;

        // Spawn endpoint entity
        self.commands
            .get_or_spawn(endpoint_id)
            .insert(endpoint);

        Ok(endpoint_id)
    }

    fn open_endpoint_inner(
        &mut self,
        entity_id: Entity,
        address: impl ToSocketAddrs,
    ) -> Result<Endpoint> {
        // Resolve address and create endpoint
        let endpoint = Endpoint::bind(resolve_address(address)?)
            .context("Failed to create a new Endpoint")?;
        let address = endpoint.address();

        // Log endpoint creation
        tracing::debug!("Opened Endpoint {entity_id:?} bound to address {address:?}");

        // Return endpoint
        Ok(endpoint)
    }

    /// Opens a connection to `address` routed through `endpoint`.
    pub fn open_connection(
        &mut self,
        address: impl ToSocketAddrs,
        endpoint: Entity,
    ) -> Result<Entity> {
        let mut endpoint_ref = self.endpoints.get_mut(endpoint)
            .context("Endpoint wasn't available to query. Maybe it was still in a command buffer.")?;

        Self::open_connection_inner(
            &mut self.commands,
            address,
            endpoint,
            &mut endpoint_ref
        )
    }

    fn open_connection_inner(
        commands: &mut Commands,
        address: impl ToSocketAddrs,
        endpoint_id: Entity,
        endpoint_ref: &mut Endpoint,
    ) -> Result<Entity> {
        // Resolve address
        let address = resolve_address(address)?;

        // Spawn connection entity
        let id = commands.spawn(/* OutgoingHandshake::new(endpoint_id, address) */ ()).id();

        // SAFETY: Commands generates a unique ID concurrently, so this is fine.
        let token = unsafe { ConnectionOwnershipToken::new(id) };
        endpoint_ref.add_peer(address, token);

        // Log connection creation
        tracing::debug!("Trying to connect to address {address:?} using id {id:?} on endpoint {endpoint_id:?}");

        Ok(id)
    }

    /// Opens an endpoint and tries to connect to a remote peer from it.
    /// Returns the entity ID of the endpoint and the entity ID of the connection.
    pub fn open_endpoint_and_connect(
        &mut self,
        address: impl ToSocketAddrs,
        remote: impl ToSocketAddrs,
    ) -> Result<(Entity, Entity)> {
        // Create endpoint, but don't actually spawn an entity
        let endpoint_id = self.entities.reserve_entity();
        let mut endpoint = self.open_endpoint_inner(endpoint_id, address)?;
        endpoint.close_on_empty = true;

        // Create connection and spawn an entity for it
        let connection_id = Self::open_connection_inner(
            &mut self.commands,
            remote,
            endpoint_id,
            &mut endpoint
        )?;

        // Spawn the endpoint entity
        // We only do this here because by this point we cannot fail
        // and so we don't have to clean up any entities we've spawned
        self.commands.get_entity(endpoint_id).unwrap().insert(endpoint);

        Ok((endpoint_id, connection_id))
    }
}

fn resolve_address(address: impl ToSocketAddrs) -> Result<SocketAddr> {
    Ok(address
    .to_socket_addrs()?
    .next()
    .ok_or_else(|| {
        anyhow::anyhow!("Invalid address")
    })?)
}

/// Implements [`ToSocketAddrs`], letting the OS choose the address and port used.
pub struct Unspecified;

impl ToSocketAddrs for Unspecified {
    type Iter = std::option::IntoIter<SocketAddr>;

    #[inline]
    fn to_socket_addrs(&self) -> std::io::Result<Self::Iter> {
        Ok(Some(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 0))).into_iter())
    }
}