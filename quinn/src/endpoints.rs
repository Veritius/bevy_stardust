use std::{collections::BTreeMap, io::ErrorKind, net::{SocketAddr, UdpSocket}, time::Instant};
use bevy::prelude::*;
use bytes::BytesMut;
use quinn_proto::{ConnectionHandle, EndpointEvent};
use crate::{connections::token::ConnectionOwnershipToken, Connection};

/// A QUIC endpoint using `quinn_proto`.
/// 
/// # Safety
/// This component must not be moved from the [`World`] it was originally added to.
#[derive(Component)]
pub struct Endpoint {
    /// The size of a buffer allocated to receive datagrams.
    /// Higher values allow remote peers to send data more efficiently.
    /// 
    /// The amount of space allocated, in bytes, is equal to the value of this field.
    /// 
    /// If this is set to below `1280`, QUIC packets may be cut off and become unreadable.
    /// Most operating systems also do not buffer UDP datagrams bigger than `65535` bytes,
    /// so setting this field that high may simply waste memory, depending on the operating system.
    pub recv_buf_size: u16,

    socket: UdpSocket,

    quinn: quinn_proto::Endpoint,

    connections: BTreeMap<ConnectionHandle, ConnectionOwnershipToken>,

    #[cfg(debug_assertions)]
    world: bevy::ecs::world::WorldId,
}

impl Endpoint {
    /// Returns the local address of the [`Endpoint`].
    pub fn local_addr(&self) -> SocketAddr {
        self.socket.local_addr().unwrap()
    }

    pub(crate) fn remove_connection(&mut self, handle: ConnectionHandle) {
        self.disassociate(handle);
        self.quinn.handle_event(handle, EndpointEvent::drained());
    }

    fn disassociate(&mut self, handle: ConnectionHandle) {
        self.connections.remove(&handle);
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
            match endpoint.socket.recv_from(&mut buf) {
                Ok((length, address)) => {
                    // Log the datagram being received
                    let trace_span = trace_span!("Received datagram", length, address=?address);
                    let _entered = trace_span.entered();

                    // Hand the datagram to Quinn
                    if let Some(event) = endpoint.quinn.handle(
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

pub(crate) fn event_exchange_system(
    mut endpoints: Query<&mut Endpoint>,
    connections: Query<&mut Connection>,
) {
    endpoints.par_iter_mut().for_each(|mut endpoint| {
        // Reborrows because borrowck angy
        let endpoint = &mut *endpoint;
        let quinn = &mut endpoint.quinn;
        let cset = &endpoint.connections;

        // Iterator over all connections the endpoint 'owns'
        let iter = cset.iter()
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
                if let Some(event) = quinn.handle_event(handle, event) {
                    connection.handle_event(event);
                }
            }
        }
    });
}

#[cfg(debug_assertions)]
pub(crate) fn safety_check_system(
    mut tokens: Local<std::collections::BTreeSet<Entity>>,
    world: bevy::ecs::world::WorldId,
    endpoints: Query<&Endpoint>,
) {
    for endpoint in &endpoints {
        assert_eq!(endpoint.world, world,
            "An Endpoint had a world ID different from the one it was created in. This is undefined behavior!");

        for connection in endpoint.connections.values() {
            assert!(!tokens.insert(connection.inner()), 
                "Two ConnectionOwnershipTokens existed simultaneously. This is undefined behavior!");
        }
    }

    tokens.clear();
}