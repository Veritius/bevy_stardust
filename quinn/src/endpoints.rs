use std::{collections::BTreeMap, io::ErrorKind, net::{SocketAddr, ToSocketAddrs, UdpSocket}, time::Instant};
use anyhow::{anyhow, Result};
use bevy_ecs::{component::{ComponentHooks, StorageType}, prelude::*};
use bytes::BytesMut;
use quinn_proto::{ClientConfig, ConnectionHandle, EndpointEvent};
use crate::{config::SocketConfig, connections::ConnectionComp};

/// A QUIC endpoint using `quinn_proto`.
pub(crate) struct EndpointComp {
    pub socket_cfg: SocketConfig,

    connections: BTreeMap<ConnectionHandle, Entity>,

    inner: Box<EndpointInner>,
}

impl Component for EndpointComp {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_remove(|mut world, entity, _| {
            // Get the component from the world
            let this = &mut *world.get_mut::<EndpointComp>(entity).unwrap();

            // Discard each connection
            for handle in this.connections.keys() {
                this.inner.detach(*handle);
            }

            // Iterator over all connections
            let ids = this.connections.values().cloned().collect::<Box<[_]>>();

            // Remove the connection component from each connection
            for id in ids.iter() {
                world.commands().entity(*id).remove::<ConnectionComp>();
            }
        });
    }
}

impl EndpointComp {
    /// Returns the local address of the [`Endpoint`].
    /// 
    /// This is the address of the local socket, and not the address that people over WAN will use to reach this endpoint.
    pub fn local_addr(&self) -> SocketAddr {
        self.inner.socket.local_addr().unwrap()
    }

    pub(crate) fn new(
        socket: UdpSocket,
        quinn: quinn_proto::Endpoint,
    ) -> Self {
        Self {
            socket_cfg: SocketConfig::default(),

            connections: BTreeMap::new(),

            inner: EndpointInner::new(
                socket,
                quinn
            ),
        }
    }

    pub(crate) fn connect(
        &mut self,
        client_config: ClientConfig,
        remote_address: impl ToSocketAddrs,
        server_name: &str,
    ) -> Result<ConnectionComp> {
        let (handle, quinn) = self.inner.quinn.connect(
            Instant::now(),
            client_config,
            remote_address.to_socket_addrs()?.next().ok_or_else(|| anyhow!("No valid connections"))?,
            server_name,
        )?;

        let mut comp = ConnectionComp::new(handle, quinn);
        todo!();

        return Ok(comp);
    }

    pub(crate) fn detach(&mut self, handle: ConnectionHandle) {
        self.connections.remove(&handle);
        self.inner.quinn.handle_event(handle, EndpointEvent::drained());
    }
}

struct EndpointInner {
    socket: UdpSocket,

    quinn: quinn_proto::Endpoint,
}

impl EndpointInner {
    fn new(
        socket: UdpSocket,
        quinn: quinn_proto::Endpoint,
    ) -> Box<Self> {
        Box::new(Self {
            socket,
            quinn,
        })
    }

    pub(crate) fn detach(&mut self, handle: ConnectionHandle) {
        self.quinn.handle_event(handle, EndpointEvent::drained());
    }
}

pub(crate) fn udp_recv_system(
    mut endpoints: Query<&mut EndpointComp>,
    mut connections: Query<&mut ConnectionComp>,
) {
    for mut endpoint in &mut endpoints {
        // Buffer for I/O operations
        debug_assert!(endpoint.socket_cfg.recv_buf_size > 1280, "Receive buffer was too small");
        let mut buf = vec![0u8; endpoint.socket_cfg.recv_buf_size as usize];

        // Store some things ahead of time
        let local_ip = endpoint.local_addr().ip();

        loop {
            match endpoint.inner.socket.recv_from(&mut buf) {
                Ok((length, address)) => {
                    #[cfg(feature="log")] {
                        // Log the datagram being received
                        let trace_span = bevy_log::trace_span!("Received datagram", length, address=?address);
                        let _entered = trace_span.entered();
                    }

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
                                let entity = endpoint.connections.get(&handle)
                                    .expect("Quic state machine returned connection handle that wasn't present in the map");

                                // SAFETY: This is a unique borrow as ConnectionOwnershipToken is unique
                                let mut connection = match connections.get_mut(*entity) {
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
    }
}

pub(crate) fn udp_send_system(
    mut endpoints: Query<&mut EndpointComp>,
    mut connections: Query<&mut ConnectionComp>,
) {
    for endpoint in &mut endpoints {
        // Buffer for I/O operations
        debug_assert!(endpoint.socket_cfg.send_buf_size > 1280, "Transmit buffer was too small");
        let mut buf = vec![0u8; endpoint.socket_cfg.send_buf_size as usize];

        // Iterate over connections
        for entity in endpoint.connections.values() {
            let mut connection = connections.get_mut(*entity).unwrap();

            // Get as many datagrams as possible
            while let Some(transmit) = connection.poll_transmit(&mut buf) {
                match endpoint.inner.socket.send_to(
                    &buf[..transmit.size],
                    transmit.destination,
                ) {
                    // Success: do nothing
                    Ok(_) => {
                        #[cfg(feature="log")]
                        bevy_log::trace!(addr=?transmit.destination, len=transmit.size, "Sent a packet");
                    },

                    Err(_) => todo!(),
                }
            }
        }
    }
}

pub(crate) fn event_exchange_system(
    mut endpoints: Query<&mut EndpointComp>,
    mut connections: Query<&mut ConnectionComp>,
) {
    for mut endpoint in &mut endpoints {
        // Reborrows because borrowck angy
        let endpoint = &mut *endpoint;

        // Exchange events
        for (handle, entity) in endpoint.connections.iter() {
            let mut connection = connections.get_mut(*entity).unwrap();

            // Timeouts can produce additional events
            connection.handle_timeout();

            // Poll until we run out of events
            while let Some(event) = connection.poll_endpoint_events() {
                if let Some(event) = endpoint.inner.quinn.handle_event(*handle, event) {
                    connection.handle_event(event);
                }
            }
        }
    }
}