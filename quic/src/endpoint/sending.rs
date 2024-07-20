use std::net::{SocketAddr, UdpSocket};
use anyhow::Result;
use bevy::{ecs::query::QueryData, prelude::Query};
use bevy_stardust::{connections::PeerMessages, messages::Outgoing};
use crate::{backend::QuicBackend, connection::Connection, ConnectionShared};
use super::scoping::{Connections, ScopedId};

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
    query: &'a Query<'a, 'a, SendConnectionsQueryData<'static, Backend>>,
}

impl<'a, Backend: QuicBackend> SendConnections<'a, Backend> {
    pub fn get_mut(&mut self, id: ScopedId<'a>) -> Option<SendConnectionHandle<Backend>> {
        // SAFETY: Using a ScopedId ensures we don't have aliasing mutable accesses
        let item = unsafe { self.query.get_unchecked(id.inner()).ok()? };

        return Some(SendConnectionHandle {
            shared: item.shared.into_inner(),
            backend: item.state.into_inner().inner(),
            messages: item.messages,
        });
    }
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