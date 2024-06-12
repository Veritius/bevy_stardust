use std::{io::ErrorKind, net::{SocketAddr, UdpSocket}, sync::Arc, time::Instant};
use anyhow::Result;
use bevy::{ecs::entity::EntityHashMap, prelude::*, utils::HashMap};
use bytes::BytesMut;
use quinn_proto::{ClientConfig, ConnectionHandle, DatagramEvent, Endpoint, EndpointConfig, ServerConfig};
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
    pub(crate) entities: HashMap<ConnectionHandle, Entity>,
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
            entities: HashMap::default(),
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

        // Add the entity ids to the map
        self.handles.insert(entity, handle);
        self.entities.insert(handle, entity);

        // Return the entity id
        return Ok(entity);
    }
}

pub(crate) fn endpoint_datagram_recv_system(
    mut endpoints: Query<(Entity, &mut QuicEndpoint)>,
    mut connections: Query<&mut QuicConnection>,
) {
    endpoints.par_iter_mut().for_each(|(entity, mut endpoint)| {
        // Logging stuff
        let trace_span = trace_span!("Reading packets from endpoint", endpoint=?entity);
        let _entered = trace_span.entered();

        // Some stuff related to the endpoint
        let endpoint = endpoint.as_mut();
        let inner = endpoint.inner.as_mut();
        let socket = &mut endpoint.socket;
        let handles = &mut endpoint.handles;
        let entities = &mut endpoint.entities;
        let ip = socket.local_addr().unwrap().ip();

        // Allocate a buffer to store messages in
        let mut buf = Vec::with_capacity(2048); // TODO: Make this based on MTU

        // Repeatedly receive messages until we run out
        loop { match socket.recv_from(&mut buf) {
            // Received another packet
            Ok((len, addr)) => {
                // Log packet receive
                trace!("Received packet of length {len} from {addr}");

                // Pass packet to the endpoint
                match inner.handle(
                    Instant::now(),
                    addr,
                    Some(ip),
                    None,
                    BytesMut::from(&buf[..]),
                    &mut buf,
                ) {
                    // Event received
                    Some(event) => match event {
                        // Connection event, route to an entity
                        DatagramEvent::ConnectionEvent(handle, event) => {
                            // Get the entity ID from the map
                            let entity = match entities.get(&handle) {
                                Some(v) => *v,
                                None => {
                                    error!("Connection {handle:?} had no associated entity");
                                    todo!(); // continue;
                                },
                            };
                        },

                        // New connection
                        DatagramEvent::NewConnection(_) => todo!(),

                        // Endpoint wants to send
                        DatagramEvent::Response(_) => todo!(),
                    },

                    // Nothing happened.
                    None => { continue },
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