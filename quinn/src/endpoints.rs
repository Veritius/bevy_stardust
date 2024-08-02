use std::{collections::BTreeMap, io::ErrorKind, net::{SocketAddr, UdpSocket}, time::Instant};
use bevy::prelude::*;
use bytes::BytesMut;
use quinn_proto::{ConnectionHandle, EndpointEvent};
use crate::Connection;

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

    connections: BTreeMap<ConnectionHandle, Entity>,
}

impl EndpointInner {
    fn new(
        socket: UdpSocket,
        quinn: quinn_proto::Endpoint,
    ) -> Box<Self> {
        Box::new(Self {
            socket,
            quinn,

            connections: BTreeMap::new(),
        })
    }
}

pub(crate) fn udp_recv_system(
    mut endpoints: Query<&mut Endpoint>,
    mut connections: Query<&mut Connection>,
) {
    for mut endpoint in &mut endpoints {
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
    mut endpoints: Query<&mut Endpoint>,
    mut connections: Query<&mut Connection>,
) {
    for endpoint in &mut endpoints {
        // Buffer for I/O operations
        debug_assert!(endpoint.send_buf_size < 1280, "Transmit buffer was too small");
        let mut buf = vec![0u8; endpoint.send_buf_size as usize];

        // Iterate over connections
        for entity in endpoint.inner.connections.values() {
            let mut connection = connections.get_mut(*entity).unwrap();

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
    }
}

pub(crate) fn event_exchange_system(
    mut endpoints: Query<&mut Endpoint>,
    mut connections: Query<&mut Connection>,
) {
    for mut endpoint in &mut endpoints {
        // Reborrows because borrowck angy
        let endpoint = &mut *endpoint;

        // Exchange events
        for (handle, entity) in endpoint.inner.connections.iter() {
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