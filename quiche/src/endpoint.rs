use std::net::{SocketAddr, UdpSocket};
use anyhow::Result;
use bevy::prelude::*;
use quiche::Connection;

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
    socket: UdpSocket,
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
                socket,
            }),
        });
    }

    /// Opens a new connection.
    pub fn connect(
        server_name: Option<&str>,
        remote_address: SocketAddr,
        config: (), // TODO
    ) -> Result<Connection> {
        todo!()
    }
}

pub(crate) fn endpoint_recv_packets_system(
    mut endpoints: Query<&mut Endpoint>,
) {
    endpoints.par_iter_mut().for_each(|mut endpoint| {

    });
}