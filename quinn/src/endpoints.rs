use std::{collections::BTreeMap, io::ErrorKind, net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket}, sync::Arc, time::Instant};
use bevy::prelude::*;
use bytes::BytesMut;
use quinn_proto::{ClientConfig, ConnectError, ConnectionHandle, EndpointConfig, EndpointEvent, ServerConfig};
use crate::{connections::{token::ConnectionOwnershipToken, ConnectionMetadata}, manager::QuinnManager, Connection};

/// A QUIC endpoint using `quinn_proto`.
/// 
/// # Safety
/// An [`Endpoint`] component being removed from the [`World`] it was created in,
/// then being added to a different [`World`], is undefined behavior.
#[derive(Component, Reflect)]
#[reflect(from_reflect=false, Component)]
pub struct Endpoint {
    /// The size of the buffer allocated to receive datagrams.
    /// Higher values allow remote peers to send data more efficiently.
    /// 
    /// The amount of space allocated, in bytes, is equal to the value of this field.
    /// 
    /// If this is set to below `1280`, QUIC packets may be cut off and become unreadable.
    /// Most operating systems also do not buffer UDP datagrams bigger than `65535` bytes,
    /// so setting this field that high may simply waste memory, depending on the operating system.
    pub recv_buf_size: u16,

    /// The size of the buffer allocated to transmit datagrams.
    /// Higher values allow more efficient transmission of information.
    /// 
    /// The amount of space allocated, in bytes, is equal to the value of this field.
    /// 
    /// If this is set to below `1280`, QUIC packets may be cut off and become unreadable.
    /// Most operating systems also do not buffer UDP datagrams bigger than `65535` bytes,
    /// so setting this field that high may simply waste memory, depending on the operating system.
    pub send_buf_size: u16,

    #[reflect(ignore)]
    inner: Box<EndpointInner>,
}

impl Endpoint {
    /// Returns the local address of the [`Endpoint`].
    /// 
    /// This is the address of the local socket, and not the address that people over WAN will use to reach this endpoint.
    pub fn local_addr(&self) -> SocketAddr {
        self.inner.socket.local_addr().unwrap()
    }
}

impl Endpoint {
    /// Create a new Quinn endpoint.
    /// 
    /// If successful, returns the local address the endpoint is bound to.
    /// 
    /// ## Servers
    /// If `server_config` is `None`, the endpoint will not be able to act as a server.
    /// The server config can be added or replaced at any time by using [`set_server_config`](Endpoint::set_server_config).
    /// 
    /// Endpoints will always be able to act as a client, through the [`connect`](Endpoint::connect) method.
    /// 
    /// ## Binding
    /// If `bind_address` is `None`, the OS will automatically assign an address to the socket.
    /// This is useful for clients, which don't need to have a known IP/port, but can make servers
    /// unreachable, such as in cases where port forwarding is needed.
    /// 
    /// If there is already a socket at the given address, `Err` is returned.
    pub fn new(
        &mut self,
        manager: &mut QuinnManager,
        quic_config: Arc<EndpointConfig>,
        server_config: Option<Arc<ServerConfig>>,
        bind_address: Option<SocketAddr>,
    ) -> anyhow::Result<SocketAddr> {
        // Giving this address to the OS means it assigns one for us
        const UNSPECIFIED: SocketAddr = SocketAddr::new(
            IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            0
        );

        // Bind and configure the socket
        let socket = UdpSocket::bind(bind_address.unwrap_or(UNSPECIFIED))?;
        socket.set_nonblocking(true)?;

        // Build the inner endpoint
        let quinn = quinn_proto::Endpoint::new(
            quic_config,
            server_config,
            true,
            None,
        );

        // Spawn a new entity with no components, we'll add them later
        let entity = manager.commands.spawn_empty().id();

        // Endpoint metadata, mostly debug stuff
        let meta = EndpointMetadata {
            eid: entity,

            #[cfg(debug_assertions)]
            world: manager.world,
        };

        // Create the endpoint and fetch some data as we're going to lose ownership soon
        let endpoint = Endpoint::new_inner(socket, quinn, meta);
        let address = endpoint.local_addr();

        // Insert the endpoint component into our new entity
        manager.commands.entity(entity).insert(endpoint);

        // Return the address
        return Ok(address);
    }

    pub(crate) fn new_inner(
        socket: UdpSocket,
        quinn: quinn_proto::Endpoint,
        meta: EndpointMetadata,
    ) -> Self {
        Self {
            recv_buf_size: 1280,
            send_buf_size: 1280,

            inner: EndpointInner::new(
                socket,
                quinn,
                meta,
            ),
        }
    }

    /// Opens a connection.
    /// 
    /// The error case occurs if the client or related parameters are misconfigured.
    /// At the point of running this, the endpoint cannot
    pub fn connect(
        &mut self,
        manager: &mut QuinnManager,
        config: ClientConfig,
        remote: SocketAddr,
        server_name: &str,
    ) -> anyhow::Result<Entity> {
        let (handle, quinn) = self.inner.quinn.connect(
            Instant::now(),
            config,
            remote,
            server_name
        )?;

        let meta = ConnectionMetadata {
            endpoint: self.meta().eid,
            handle,

            #[cfg(debug_assertions)]
            world: manager.world,
        };

        let id = manager.commands.spawn(Connection::new(quinn, meta)).id();

        // SAFETY: We just spawned this entity
        let token = unsafe { ConnectionOwnershipToken::new(id) };
        self.inner.connections.insert(handle, token);

        return Ok(id);
    }

    pub(crate) fn meta(&self) -> &EndpointMetadata {
        &self.inner.meta
    }

    pub(crate) fn remove_connection(&mut self, handle: ConnectionHandle) {
        self.disassociate(handle);
        self.inner.quinn.handle_event(handle, EndpointEvent::drained());
    }

    fn disassociate(&mut self, handle: ConnectionHandle) {
        self.inner.connections.remove(&handle);
    }
}

struct EndpointInner {
    socket: UdpSocket,

    quinn: quinn_proto::Endpoint,

    connections: BTreeMap<ConnectionHandle, ConnectionOwnershipToken>,

    meta: EndpointMetadata,
}

impl EndpointInner {
    fn new(
        socket: UdpSocket,
        quinn: quinn_proto::Endpoint,
        meta: EndpointMetadata,
    ) -> Box<Self> {
        Box::new(Self {
            socket,
            quinn,

            connections: BTreeMap::new(),

            meta,
        })
    }
}

pub(crate) fn udp_recv_system(
    mut endpoints: Query<&mut Endpoint>,
    connections: Query<&mut Connection>,
) {
    endpoints.par_iter_mut().for_each(|mut endpoint| {
        // Buffer for I/O operations
        debug_assert!(endpoint.recv_buf_size < 1280, "Receive buffer was too small");
        let mut buf = vec![0u8; endpoint.recv_buf_size as usize];

        // Store some things ahead of time
        let local_ip = endpoint.local_addr().ip();

        loop {
            match endpoint.inner.socket.recv_from(&mut buf) {
                Ok((length, address)) => {
                    // Log the datagram being received
                    let trace_span = trace_span!("Received datagram", length, address=?address);
                    let _entered = trace_span.entered();

                    // Hand the datagram to Quinn
                    if let Some(event) = endpoint.inner.quinn.handle(
                        Instant::now(),
                        address,
                        Some(local_ip),
                        None, // TODO
                        BytesMut::from(&buf[..length]),
                        &mut buf,
                    ) {
                        match event {
                            // Event for an existing connection
                            quinn_proto::DatagramEvent::ConnectionEvent(handle, event) => {
                                // Get the entity from the handle, which we need to access the connection
                                let entity = endpoint.inner.connections.get(&handle)
                                    .expect("Quic state machine returned connection handle that wasn't present in the map");

                                // SAFETY: This is a unique borrow as ConnectionOwnershipToken is unique
                                let mut connection = match unsafe { connections.get_unchecked(entity.inner()) } {
                                    Ok(v) => v,
                                    Err(_) => todo!(),
                                };

                                // Handle the event
                                connection.handle_event(event);
                            },

                            // A new connection can potentially be established
                            quinn_proto::DatagramEvent::NewConnection(incoming) => {
                                todo!()
                            },

                            // Immediate response
                            quinn_proto::DatagramEvent::Response(transmit) => {
                                todo!()
                            },
                        }
                    }
                },

                // If this occurs, it means there are no more packets to read
                Err(e) if e.kind() == ErrorKind::WouldBlock => break,

                Err(e) => todo!(),
            }
        }
    });
}

pub(crate) fn udp_send_system(
    mut endpoints: Query<&mut Endpoint>,
    connections: Query<&mut Connection>,
) {
    endpoints.par_iter_mut().for_each(|mut endpoint| {
        // Buffer for I/O operations
        debug_assert!(endpoint.send_buf_size < 1280, "Transmit buffer was too small");
        let mut buf = vec![0u8; endpoint.send_buf_size as usize];

        // Reborrows because borrowck angy
        let endpoint = &mut *endpoint;

        // Iterator over all connections the endpoint 'owns'
        let iter = endpoint.inner.connections.values()
            .map(|token| {
                // SAFETY: We know this borrow is unique because ConnectionOwnershipToken is unique
                unsafe { connections.get_unchecked(token.inner()).unwrap() }
            });

        // Iterate over connections
        for mut connection in iter {
            // Get as many datagrams as possible
            while let Some(transmit) = connection.poll_transmit(&mut buf) {
                match endpoint.inner.socket.send_to(
                    &buf[..transmit.size],
                    transmit.destination,
                ) {
                    // Success: do nothing
                    Ok(_) => {},

                    Err(_) => todo!(),
                }
            }
        }
    });
}

pub(crate) fn event_exchange_system(
    mut endpoints: Query<&mut Endpoint>,
    connections: Query<&mut Connection>,
) {
    endpoints.par_iter_mut().for_each(|mut endpoint| {
        // Reborrows because borrowck angy
        let endpoint = &mut *endpoint;

        // Iterator over all connections the endpoint 'owns'
        let iter = endpoint.inner.connections.iter()
            .map(|(handle, token)| {
                // SAFETY: We know this borrow is unique because ConnectionOwnershipToken is unique
                let c = unsafe { connections.get_unchecked(token.inner()).unwrap() };
                (*handle, c)
            });

        // Exchange events
        for (handle, mut connection) in iter {
            // Timeouts can produce additional events
            connection.handle_timeout();

            // Poll until we run out of events
            while let Some(event) = connection.poll_endpoint_events() {
                if let Some(event) = endpoint.inner.quinn.handle_event(handle, event) {
                    connection.handle_event(event);
                }
            }
        }
    });
}

pub(crate) struct EndpointMetadata {
    pub eid: Entity,

    #[cfg(debug_assertions)]
    pub world: bevy::ecs::world::WorldId,
}

#[cfg(debug_assertions)]
pub(crate) fn safety_check_system(
    mut tokens: Local<std::collections::BTreeSet<Entity>>,
    world: bevy::ecs::world::WorldId,
    endpoints: Query<&Endpoint>,
) {
    for endpoint in &endpoints {
        assert_eq!(world, endpoint.inner.meta.world,
            "An Endpoint was moved from the world it was originally added to. This is undefined behavior!");

        for connection in endpoint.inner.connections.values() {
            assert!(!tokens.insert(connection.inner()), 
                "Two ConnectionOwnershipTokens existed simultaneously. This is undefined behavior!");
        }
    }

    tokens.clear();
}