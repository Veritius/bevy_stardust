use std::{collections::BTreeMap, io::ErrorKind, net::{SocketAddr, UdpSocket}};
use anyhow::Result;
use bevy::prelude::*;
use crate::{events::{event_pair, EndpointEvents}, Connection};

/// A QUIC endpoint.
#[derive(Component, Reflect)]
#[reflect(from_reflect = false, Component)]
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

struct EndpointInner {
    address: SocketAddr,
    socket: UdpSocket,

    connections: BTreeMap<SocketAddr, Box<ConnectionHandle>>,
}

impl Endpoint {
    /// Creates a new [`Endpoint`].
    pub fn new(
        bind_address: SocketAddr,
    ) -> Result<Self> {
        // Bind the socket
        let socket = UdpSocket::bind(bind_address)?;
        socket.set_nonblocking(true)?;

        // Return component
        return Ok(Self {
            recv_buf_size: 1280,
            send_buf_size: 1280,

            inner: Box::new(EndpointInner {
                address: bind_address,
                socket,

                connections: BTreeMap::new(),
            }),
        });
    }

    /// Opens a new connection.
    pub fn connect(
        &mut self,
        server_name: Option<&str>,
        remote_address: SocketAddr,
        config: (), // TODO
    ) -> Result<Connection> {
        // Create the Quiche connection
        let qcon = quiche::connect(
            server_name,
            todo!(),
            self.inner.address,
            remote_address,
            todo!(),
        )?;

        // Create the event messaging pairs
        let (con_evts, ept_evts) = event_pair();

        // Register the connection to the endpoint
        self.inner.connections.insert(remote_address, Box::new(ConnectionHandle {
            events: ept_evts,
        }));

        // Return the new connection
        return Ok(Connection::new(
            remote_address,
            qcon,
            con_evts,
        ));
    }
}

struct ConnectionHandle {
    events: EndpointEvents,
}

pub(crate) fn endpoint_recv_packets_system(
    commands: ParallelCommands,
    mut endpoints: Query<&mut Endpoint>,
) {
    endpoints.par_iter_mut().for_each(|mut endpoint| {
        let mut buf = vec![0u8; endpoint.recv_buf_size.into()];
        debug_assert_eq!(buf.len(), buf.capacity());

        loop {
            match endpoint.inner.socket.recv_from(&mut buf) {
                // A packet was received
                Ok((len, addr)) => {
                    // If the packet is too short, it's not valid QUIC, so we ignore it
                    if len < 1200 { continue }

                    // Create a slice of the received data
                    let slice = &buf[..len];

                    // Check if this peer already exists
                    match endpoint.inner.connections.get_mut(&addr) {
                        // This peer exists, yay!
                        Some(handle) => {
                            // Try to inform the peer of its existence
                            if let Err(error) = handle.events.try_send_payload(slice) {
                                todo!()
                            }
                        },

                        // Peer does not exist, try to create it
                        None => {
                            match quiche::accept(
                                todo!(),
                                todo!(),
                                endpoint.inner.address,
                                addr,
                                todo!(),
                            ) {
                                Ok(qcon) => {
                                    // Create the event messaging pairs
                                    let (con_evts, ept_evts) = event_pair();
                                    ept_evts.try_send_payload(slice).unwrap(); // We just created it, so it can't be closed

                                    // Spawn a new connection entity
                                    commands.command_scope(|mut commands| {
                                        commands.spawn(Connection::new(
                                            addr,
                                            qcon,
                                            con_evts,
                                        ));
                                    });

                                    // Add the connection to the endpoint
                                    endpoint.inner.connections.insert(addr, Box::new(ConnectionHandle {
                                        events: ept_evts,
                                    }));
                                },

                                Err(_) => todo!(),
                            }
                        },
                    }
                },

                // Occurs when there are no more packets to read
                Err(e) if e.kind() == ErrorKind::WouldBlock => {
                    todo!()
                }

                Err(_) => todo!(),
            }
        }
    });
}