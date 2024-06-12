use std::{io::ErrorKind, net::{SocketAddr, UdpSocket}, sync::Arc, time::Instant};
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
    pub(crate) socket: UdpSocket,
}

impl QuicEndpoint {
    /// Creates a new endpoint, bound to `socket`.
    /// 
    /// If `server_config` is `None`, incoming connections will be rejected.
    /// This can be changed at any time with [`set_server_config`](Self::set_server_config).
    pub fn new(
        socket: UdpSocket,
        config: Arc<EndpointConfig>,
        server_config: Option<Arc<ServerConfig>>,
        allow_mtud: bool,
        rng_seed: Option<[u8; 32]>,
    ) -> Result<Self> {
        // Sockets must be nonblocking
        socket.set_nonblocking(true)?;

        Ok(Self {
            inner: Box::new(Endpoint::new(config, server_config, allow_mtud, rng_seed)),
            handles: EntityHashMap::default(),
            socket,
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

pub(crate) fn endpoint_datagram_recv_system(
    mut endpoints: Query<&mut QuicEndpoint>,
    mut connections: Query<&mut QuicConnection>,
) {
    endpoints.par_iter_mut().for_each(|mut endpoint| {
        // Allocate a buffer to store messages in
        let mut buf = Vec::with_capacity(2048); // TODO: Make this based on MTU

        // Repeatedly receive messages until we run out
        loop { match endpoint.socket.recv_from(&mut buf) {
            // Received another packet
            Ok((len, addr)) => {
                match endpoint.inner.handle(
                    Instant::now(),
                    addr,
                    Some(endpoint.socket.local_addr().unwrap().ip()),
                    None,
                    todo!(),
                    &mut buf,
                ) {
                    Some(_) => todo!(),
                    None => todo!(),
                }
            },

            // There are no more packets to read, break the loop
            Err(e) if e.kind() == ErrorKind::WouldBlock => { break },

            // An actual IO error occurred
            Err(e) => {
                todo!()
            },
        }}
    });
}