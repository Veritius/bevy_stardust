use std::{io::ErrorKind, net::{SocketAddr, UdpSocket}};
use anyhow::Result;
use bevy::ecs::query::QueryData;
use bevy_stardust::{connections::PeerMessages, messages::Incoming};
use crate::{backend::QuicBackend, connection::Connection, ConnectionShared};
use super::scoping::{Connections, ScopedAccess};

/// A handle to a UDP socket.
pub struct UdpSocketRecv<'a> {
    socket: &'a UdpSocket,
    scratch: &'a mut [u8],
}

impl<'a> UdpSocketRecv<'a> {
    pub fn recv(&mut self) -> Result<Option<ReceivedDatagram>> {
        match self.socket.recv_from(&mut self.scratch) {
            Ok((length, address)) => {
                let payload = &self.scratch[..length];
                return Ok(Some(ReceivedDatagram { address, payload }));
            },

            Err(err) if err.kind() == ErrorKind::WouldBlock => return Ok(None),

            Err(err) => return Err(<std::io::Error as Into<anyhow::Error>>::into(err)
                .context("while receiving udp packets")),
        }
    }
}

/// A datagram that has been received.
pub struct ReceivedDatagram<'a> {
    pub address: SocketAddr,
    pub payload: &'a [u8],
}

pub struct RecvConnections<'a, Backend: QuicBackend> {
    connections: Connections<'a>,
    query_data: ScopedAccess<'a, RecvConnectionsQueryData<'a, Backend>>,
}

pub struct RecvConnectionHandle<'a, Backend: QuicBackend> {
    shared: &'a mut ConnectionShared,
    backend: &'a mut Backend::ConnectionState,
    messages: &'a mut PeerMessages<Incoming>,
}

#[derive(QueryData)]
#[query_data(mutable)]
struct RecvConnectionsQueryData<'w, Backend: QuicBackend> {
    shared: &'w mut ConnectionShared,
    state: &'w mut Connection<Backend::ConnectionState>,
    messages: &'w mut PeerMessages<Incoming>,
}