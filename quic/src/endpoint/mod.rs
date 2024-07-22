mod connections;

use std::net::{SocketAddr, UdpSocket};
use anyhow::ensure;
use bevy::prelude::*;

pub(crate) use connections::EndpointConnections;

/// A QUIC endpoint, corresponding to a single UDP socket.
/// 
/// All [connections](crate::Connection) 'belong' to an Endpoint, which they use for I/O.
#[derive(Component)]
pub struct Endpoint {
    /// If `true`, the endpoint will listen for new, incoming connections.
    pub listening: bool,

    pub(crate) send_size: usize,
    pub(crate) recv_size: usize,

    pub(crate) socket: UdpSocket,
    pub(crate) connections: EndpointConnections,
}

impl Endpoint {
    pub(crate) fn socket(&self) -> &UdpSocket {
        &self.socket
    }
}

impl Endpoint {
    /// Returns the local address this endpoint is bound to.
    pub fn local_addr(&self) -> SocketAddr {
        self.socket.local_addr().unwrap()
    }

    /// Configures the length of the buffer allocated while receiving UDP packets.
    /// Must be at least `1280` (imposed by the QUIC standard), or an error will occur.
    pub fn set_recv_buf_len(&mut self, len: usize) -> anyhow::Result<()> {
        ensure!(len > 1280, "Length was smaller than minimum QUIC value");
        self.recv_size = len;
        
        #[cfg(feature="quiche")]
        self.quiche_config.set_max_recv_udp_payload_size(len);

        return Ok(())
    }

    /// Configures the length of the buffer allocated while sending UDP packets.
    /// Must be at least `1200` (imposed by the QUIC standard), or an error will occur.
    pub fn set_send_buf_len(&mut self, len: usize) -> anyhow::Result<()> {
        ensure!(len > 1200, "Length was smaller than minimum QUIC value");
        self.recv_size = len;

        #[cfg(feature="quiche")]
        self.quiche_config.set_max_send_udp_payload_size(len);

        return Ok(())
    }
}