// mod builder;
mod connections;
mod receiving;
mod scoping;
mod sending;
mod systems;

use std::net::{SocketAddr, UdpSocket};
use anyhow::ensure;
use bevy::prelude::*;
use crate::backend::QuicBackend;

pub(crate) use connections::EndpointConnections;

pub use receiving::{UdpSocketRecv, ReceivedDatagram, RecvConnections};
pub use sending::{UdpSocketSend, TransmitDatagram, SendConnections};

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

    fn recv<'a>(
        &'a mut self,
        backend: &'a Self::Backend,
        socket: UdpSocketRecv<'a>,
        connections: RecvConnections<'a, Self::Backend>,
    );

    fn send<'a>(
        &'a mut self,
        backend: &'a Self::Backend,
        socket: UdpSocketSend<'a>,
        connections: SendConnections<'a, Self::Backend>,
    );
}

#[derive(Component)]
pub struct Endpoint<State: EndpointState> {
    state: State,
}

impl<State: EndpointState> Endpoint<State> {
    fn inner(&self) -> &State {
        &self.state
    }

    fn inner_mut(&mut self) -> &mut State {
        &mut self.state
    }
}