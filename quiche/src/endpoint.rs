use std::net::{SocketAddr, UdpSocket};
use anyhow::Result;
use bevy::prelude::*;

/// A QUIC endpoint.
#[derive(Component, Reflect)]
#[reflect(from_reflect = false, Component)]
pub struct Endpoint {
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
            inner: Box::new(EndpointInner {
                socket,
            }),
        });
    }
}

pub(crate) fn endpoint_recv_packets_system(
    mut endpoints: Query<&mut Endpoint>,
) {
    endpoints.par_iter_mut().for_each(|mut endpoint| {

    });
}