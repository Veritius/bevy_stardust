use std::net::{SocketAddr, UdpSocket};
use anyhow::Result;
use bevy::ecs::query::QueryData;
use bevy_stardust::{connections::PeerMessages, messages::Outgoing};
use crate::{backend::QuicBackend, connection::Connection, ConnectionShared};
use super::scoping::{Connections, ScopedAccess};

/// A handle to a UDP socket.
pub struct UdpSocketSend<'a> {
    socket: &'a UdpSocket,
}

impl<'a> UdpSocketSend<'a> {
    pub fn send(&mut self, transmit: TransmitDatagram) -> Result<()> {
        match self.socket.send_to(transmit.payload, transmit.address) {
            Ok(_) => return Ok(()),
            Err(err) => return Err(err.into()),
        }
    }
}

/// A datagram that must be transmitted.
pub struct TransmitDatagram<'a> {
    pub address: SocketAddr,
    pub payload: &'a [u8],
}

pub struct SendConnections<'a, Backend: QuicBackend> {
    connections: Connections<'a>,
    query_data: ScopedAccess<'a, SendConnectionsQueryData<'a, Backend>>,
}

pub struct SendConnectionHandle<'a, Backend: QuicBackend> {
    shared: &'a mut ConnectionShared,
    backend: &'a mut Backend::ConnectionState,
    messages: &'a PeerMessages<Outgoing>,
}

#[derive(QueryData)]
#[query_data(mutable)]
struct SendConnectionsQueryData<'w, Backend: QuicBackend> {
    shared: &'w mut ConnectionShared,
    state: &'w mut Connection<Backend::ConnectionState>,
    messages: &'w PeerMessages<Outgoing>,
}