use std::{net::SocketAddr, time::Instant};
use anyhow::Result;
use bevy::prelude::*;
use quinn_proto::{ClientConfig, Connection, ConnectionHandle};
use crate::QuicEndpoint;

/// A QUIC connection, attached to an endpoint.
/// 
/// # Safety
/// This component must always stay in the same [`World`] as it was created in.
/// Being put into another `World` will lead to undefined behavior.
#[derive(Component)]
pub struct QuicConnection {
    handle: ConnectionHandle,
    inner: Box<Connection>,
}

impl QuicConnection {
    /// Initiates a connection, spawning a new entity using `commands`.
    /// The entity ID of the new connection will be returned if successful.
    pub fn connect(
        commands: &mut Commands,
        endpoint: &mut QuicEndpoint,
        config: ClientConfig,
        remote: SocketAddr,
        server_name: &str
    ) -> Result<Entity> {
        // Create the connection first, so if it fails, we don't spuriously spawn entities
        let (handle, connection) = endpoint.inner.connect(Instant::now(), config, remote, server_name)?;

        // Spawn the connection entity
        let entity = commands.spawn(Self {
            handle,
            inner: Box::new(connection),
        }).id();

        // Add to the endpoint handles set
        endpoint.handles.insert(entity, handle);

        // Return the entity id
        return Ok(entity);
    }

    /// Initiates a connection, associating with id `entity`.
    /// If the guarantees below are not followed, this function causes UB.
    /// 
    /// To use this function safely, the following guarantees must be made:
    /// - `entity` must be the entity id of the entity this component is being added to
    // SAFETY: This is unsafe due to concurrency in some systems, which can cause data races
    pub unsafe fn connect_unsafe(
        endpoint: &mut QuicEndpoint,
        entity: Entity,
        config: ClientConfig,
        remote: SocketAddr,
        server_name: &str,
    ) -> Result<Self> {
        // Create the connection and register it to the endpoint
        let (handle, connection) = endpoint.inner.connect(Instant::now(), config, remote, server_name)?;
        endpoint.handles.insert(entity, handle);

        // Return the component
        return Ok(Self {
            handle,
            inner: Box::new(connection),
        });
    }
}