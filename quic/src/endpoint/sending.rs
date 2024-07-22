use std::net::{SocketAddr, UdpSocket};
use anyhow::Result;
use bevy::{ecs::query::QueryData, prelude::*};
use bevy_stardust::{connections::PeerMessages, messages::Outgoing};
use crate::{backend::{BackendInstance, QuicBackend}, connection::ConnectionStateData, Connection, Endpoint};
use super::scoping::{Connections, ScopedId};

/// A handle to a UDP socket.
pub struct UdpSocketSend<'a> {
    socket: &'a UdpSocket,
}

impl<'a> UdpSocketSend<'a> {
    /// Try to send a UDP packet over the socket.
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
            id,
            shared: item.shared.into_inner(),
            backend: item.state.into_inner().inner(),
            messages: item.messages,
        });
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = SendConnectionHandle<Backend>> + '_{
        self.connections.iter()
            .map(|id| (id, unsafe { self.query.get_unchecked(id.inner()) }))
            .filter(|(_, item)| item.is_ok())
            .map(|(id, item)| (id, item.unwrap()))
            .map(|(id, item)| SendConnectionHandle {
                id,
                shared: item.shared.into_inner(),
                backend: item.state.into_inner().inner(),
                messages: item.messages,
            })
    }
}

pub struct SendConnectionHandle<'a, Backend: QuicBackend> {
    id: ScopedId<'a>,
    shared: &'a mut Connection,
    backend: &'a mut Backend::ConnectionState,
    messages: &'a PeerMessages<Outgoing>,
}

impl<'a, Backend: QuicBackend> SendConnectionHandle<'a, Backend> {
    pub fn id(&'a self) -> ScopedId<'a> {
        self.id
    }

    pub fn state(&'a mut self) -> &'a mut Backend::ConnectionState {
        self.backend
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
struct SendConnectionsQueryData<'w, Backend: QuicBackend> {
    shared: &'w mut Connection,
    state: &'w mut ConnectionStateData<Backend::ConnectionState>,
    messages: &'w PeerMessages<Outgoing>,
}