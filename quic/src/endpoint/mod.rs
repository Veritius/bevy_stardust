// mod builder;
mod connections;
mod receiving;
mod scoping;
mod sending;

use std::net::{SocketAddr, UdpSocket};
use anyhow::ensure;
use bevy::prelude::*;
use crate::backend::QuicBackend;

pub(crate) use connections::EndpointConnections;

// #[allow(unused)] // various backends may or may not use these exports
// pub(crate) use builder::{ReadyShared, HostShared, JoinShared, ClientReady, ServerReady, DualReady};

// pub use builder::{EndpointBuilder, Client, Server, Dual};
pub use sending::Transmit;

/// Endpoint state information.
/// 
/// All [connections](crate::Connection) 'belong' to an Endpoint, which they use for I/O.
#[derive(Component)]
pub struct EndpointShared {
    /// If `true`, the endpoint will listen for new, incoming connections.
    pub listening: bool,

    pub(crate) socket: UdpSocket,
    pub(crate) connections: EndpointConnections,

    pub(crate) send_size: usize,
    pub(crate) recv_size: usize,
}

impl EndpointShared {
    /// Returns the local address this endpoint is bound to.
    pub fn local_addr(&self) -> SocketAddr {
        self.socket.local_addr().unwrap()
    }

    /// Configures the length of the buffer allocated while receiving UDP packets.
    /// Must be at least `1280` (imposed by the QUIC standard), or an error will occur.
    pub fn set_recv_buf_len(&mut self, len: usize) -> anyhow::Result<()> {
        ensure!(len > 1280, "Length was smaller than minimum QUIC value");
        self.recv_size = len;

        return Ok(())
    }

    /// Configures the length of the buffer allocated while sending UDP packets.
    /// Must be at least `1200` (imposed by the QUIC standard), or an error will occur.
    pub fn set_send_buf_len(&mut self, len: usize) -> anyhow::Result<()> {
        ensure!(len > 1200, "Length was smaller than minimum QUIC value");
        self.recv_size = len;

        return Ok(())
    }
}

/// An endpoint associated with a [`Backend`](crate::backend::Backend) implementation.
pub trait EndpointState
where
    Self: Send + Sync,
{
    /// The [`QuicBackend`] implementation that manages this endpoint.
    type Backend: QuicBackend;

    /// Errors that can occur when using `recv_udp_packet` or `send_udp_packet`.
    type IoError: Into<anyhow::Error>;

    /// Called when a new UDP packet is received.
    /// 
    /// `from` is the IP address and port the packet was sent from.
    /// `packet` is a slice containing the full received data.
    fn recv_udp_packet(&mut self, from: SocketAddr, packet: &[u8]) -> Result<(), Self::IoError>;

    /// Called to see if the backend wants to transmit any new packets.
    fn send_udp_packet(&mut self) -> impl Iterator<Item = Result<Transmit, Self::IoError>> + '_;
}

#[derive(Component)]
pub struct Endpoint<State: EndpointState> {
    state: State,
}