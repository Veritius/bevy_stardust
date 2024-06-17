use std::{io::ErrorKind, net::{SocketAddr, UdpSocket}, sync::Arc, time::Instant};
use anyhow::Result;
use bevy::{prelude::*, utils::HashMap};
use bytes::BytesMut;
use quinn_proto::{ClientConfig, ConnectionHandle, DatagramEvent, Endpoint, EndpointConfig, ServerConfig, Transmit};
use crate::QuicConnection;

/// A QUIC endpoint.
/// 
/// # Safety
/// This component must always stay in the same [`World`] as it was created in.
/// Being put into another `World` will lead to undefined behavior.
#[derive(Component)]
pub struct QuicEndpoint {
    pub(crate) inner: Box<Endpoint>,
    pub(crate) entities: HashMap<ConnectionHandle, Entity>,
    pub(crate) socket: UdpSocket,

    listening: bool,
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

        debug!("Opened QUIC endpoint on address {:?}", socket.local_addr().unwrap());

        Ok(Self {
            inner: Box::new(Endpoint::new(config, server_config, allow_mtud, rng_seed)),
            entities: HashMap::default(),
            socket,
            listening: true,
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

    /// Sets whether or not the endpoint is listening to new connections.
    /// Only affects endpoints with a server config. Enabled by default.
    /// Turning this off does not remove the attached server config.
    pub fn set_listening(
        &mut self,
        val: bool,
    ) {
        self.listening = val;
    }

    /// Connects to a new connection.
    /// `entity` must be the id of the entity that the `QuicEndpoint` component is attached to.
    /// 
    /// # Safety
    /// `commands` must only be applied to the world the `QuicEndpoint` is part of.
    pub fn connect(
        &mut self,
        commands: &mut Commands,
        entity: Entity,
        config: ClientConfig,
        remote: SocketAddr,
        server_name: &str,
    ) -> Result<Entity> {
        // Create the connection data
        let (handle, connection) = self.inner.connect(Instant::now(), config, remote, server_name)?;

        // Spawn the connection entity
        let component = QuicConnection::new(entity, handle, Box::new(connection));
        let entity = commands.spawn(component).id();

        // Add the entity ids to the map
        self.entities.insert(handle, entity);

        // Return the entity id
        return Ok(entity);
    }
}

pub(crate) fn endpoint_datagram_recv_system(
    commands: ParallelCommands,
    mut endpoints: Query<(Entity, &mut QuicEndpoint)>,
    connections: Query<&mut QuicConnection>,
) {
    endpoints.par_iter_mut().for_each(|(entity, mut endpoint)| {
        // Logging stuff
        let trace_span = trace_span!("Reading packets from endpoint", endpoint=?entity);
        let _entered = trace_span.entered();

        // Some stuff related to the endpoint
        let endpoint = endpoint.as_mut();
        let inner = endpoint.inner.as_mut();
        let socket = &mut endpoint.socket;
        let entities = &mut endpoint.entities;
        let listening = endpoint.listening;
        let ip = socket.local_addr().unwrap().ip();

        // Allocate a buffer to store messages in
        let buf_size = 2048; // TODO: Make this based on MTU
        let mut buf = Vec::with_capacity(buf_size);
        buf.extend((0..buf_size).into_iter().map(|_| 0)); // Fill with zeros

        // Repeatedly receive messages until we run out
        loop {
            /*
                SAFETY

                This is needed because
                1. recv_from takes a mutable slice, which can't resize
                2. recv_from will drop any bytes that can't fit in the slice
                3. Without this, the slice's length may be lower than it should be
                4. Endpoint::handle may change the vec's length or capacity

                This is fine because
                1. u8s have no special drop or initialisation considerations
                2. We filled the vec with zeros, replacing old, uninitialised memory
                3. recv_from returns the valid length of actually initialised data
                4. The buffer already fills with arbitrary, untrusted data
            */
            unsafe { buf.set_len(buf.capacity()); }

            match socket.recv_from(&mut buf) {
                // Received another packet
                Ok((len, addr)) => {
                    // More logging stuff
                    let trace_span = trace_span!("Reading packet", len, address=?addr);
                    let _entered = trace_span.entered();

                    // Pass packet to the endpoint
                    match inner.handle(
                        Instant::now(),
                        addr,
                        Some(ip),
                        None,
                        BytesMut::from(&buf[..len]),
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

                                // SAFETY: Only this endpoint accesses this entity, since it controls it
                                let query_item = unsafe { connections.get_unchecked(entity) };
                                let mut connection = match query_item {
                                    Ok(v) => v,
                                    Err(_) => {
                                        error!("Failed to get connection component from {entity:?}");
                                        continue;
                                    },
                                };

                                // Connection handles event
                                connection.inner.handle_event(event);
                            },

                            // New connection
                            DatagramEvent::NewConnection(incoming) => {
                                // If the server isn't listening, immediately reject them.
                                if !listening {
                                    // Refuse the connection and send the refusal packet
                                    let transmit = inner.refuse(incoming, &mut buf);
                                    perform_transmit(socket, &buf, transmit);

                                    // Move on
                                    continue;
                                }

                                // Accept the connection immediately
                                // TODO: Allow game systems to deny the connection
                                match inner.accept(incoming, Instant::now(), &mut buf, None) {
                                    // Acceptance succeeded :)
                                    Ok((handle, connection)) => {
                                        // Spawn the connection entity
                                        let connection = Box::new(connection);
                                        let entity = commands.command_scope(move |mut commands| {
                                            let component = QuicConnection::new(
                                                entity,
                                                handle,
                                                connection
                                            );
                                            commands.spawn(component).id()
                                        });

                                        // Add the entity ids to the map
                                        entities.insert(handle, entity);
                                    },

                                    // An error occurred
                                    Err(err) => {
                                        // Log the error to the console
                                        debug!("Error occurred while accepting peer: {}", err.cause);

                                        // The error may have an associated response
                                        if let Some(transmit) = err.response {
                                            perform_transmit(socket, &buf, transmit);
                                        }

                                        // Done
                                        continue;
                                    },
                                }
                            },

                            // Endpoint wants to send
                            DatagramEvent::Response(transmit) => {
                                perform_transmit(socket, &buf, transmit);
                            },
                        },

                        // Nothing happened.
                        None => { continue },
                    }
                },

                // There are no more packets to read, break the loop
                Err(e) if e.kind() == ErrorKind::WouldBlock => { break },

                // An actual IO error occurred
                Err(e) => {
                    error!("IO error occurred while reading UDP packets: {e}");
                    break;
                },
            }
        }
    });
}

pub(crate) fn endpoint_datagram_send_system(
    mut endpoints: Query<(Entity, &mut QuicEndpoint)>,
    connections: Query<&mut QuicConnection>,
) {
    endpoints.par_iter_mut().for_each(|(entity, mut endpoint)| {
        // Logging stuff
        let trace_span = trace_span!("Sending packets from endpoint", endpoint=?entity);
        let _entered = trace_span.entered();

        // Some stuff related to the endpoint
        let endpoint = endpoint.as_mut();
        let socket = &mut endpoint.socket;

        // Allocate a buffer to store messages in
        let mut buf = Vec::with_capacity(2048); // TODO: Make this based on MTU

        // Iterate over all connections associated with this endpoint
        let entities = endpoint.entities.iter();
        for (handle, entity) in entities {
            // Logging stuff
            let trace_span = trace_span!("Polling connection", connection=?entity, handle=?handle);
            let _entered = trace_span.entered();

            // SAFETY: Endpoints will only access the connections they have created
            let query_item = unsafe { connections.get_unchecked(*entity) };
            let mut connection = match query_item {
                Ok(connection) => connection,
                Err(err) => todo!(),
            };

            // Handle timeouts
            connection.inner.handle_timeout(Instant::now());

            // Repeatedly poll transmit until the connection no longer wants to send any more packets
            let mut send_count: u32 = 0;
            while let Some(transmit) = connection.inner.poll_transmit(Instant::now(), 1, &mut buf) {
                perform_transmit(socket, &buf, transmit);
                send_count += 1;
                buf.clear(); // Clear the buffer
            }

            // Record the amount of packets we've sent
            _entered.record("sends", send_count);
        }
    });
}

fn perform_transmit(
    socket: &mut UdpSocket,
    payload: &[u8],
    transmit: Transmit,
) {
    match socket.send_to(&payload, transmit.destination) {
        Ok(len) => { debug_assert_eq!(transmit.size, len); },
        Err(err) => { error!("Error while sending packet: {err}"); },
    }
}