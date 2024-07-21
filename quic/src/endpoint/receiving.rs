use std::{io::ErrorKind, net::{SocketAddr, UdpSocket}};
use anyhow::Result;
use bevy::{ecs::query::QueryData, prelude::Query};
use bevy_stardust::{connections::PeerMessages, messages::Incoming};
use crate::{backend::QuicBackend, connection::Connection, ConnectionShared};
use super::scoping::{Connections, ScopedId};

/// A handle to a UDP socket.
pub struct UdpSocketRecv<'a> {
    socket: &'a UdpSocket,
    scratch: &'a mut [u8],
}

impl<'a> UdpSocketRecv<'a> {
    /// Try to receive packets, with three possible cases:
    /// - `Ok(Some())` - A UDP packet was received over the socket
    /// - `Ok(None)` - No more packets are available this tick
    /// - `Err()` - An I/O error occurred
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
    query: &'a Query<'a, 'a, RecvConnectionsQueryData<'static, Backend>>,
}

impl<'a, Backend: QuicBackend> RecvConnections<'a, Backend> {
    pub fn get_mut(&mut self, id: ScopedId<'a>) -> Option<RecvConnectionHandle<Backend>> {
        // SAFETY: Using a ScopedId ensures we don't have aliasing mutable accesses
        let item = unsafe { self.query.get_unchecked(id.inner()).ok()? };

        return Some(RecvConnectionHandle {
            id,
            shared: item.shared.into_inner(),
            backend: item.state.into_inner().inner(),
            messages: item.messages.into_inner(),
        });
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = RecvConnectionHandle<Backend>> + '_{
        self.connections.iter()
            .map(|id| (id, unsafe { self.query.get_unchecked(id.inner()) }))
            .filter(|(_, item)| item.is_ok())
            .map(|(id, item)| (id, item.unwrap()))
            .map(|(id, item)| RecvConnectionHandle {
                id,
                shared: item.shared.into_inner(),
                backend: item.state.into_inner().inner(),
                messages: item.messages.into_inner(),
            })
    }
}

pub struct RecvConnectionHandle<'a, Backend: QuicBackend> {
    id: ScopedId<'a>,
    shared: &'a mut ConnectionShared,
    backend: &'a mut Backend::ConnectionState,
    messages: &'a mut PeerMessages<Incoming>,
}

impl<'a, Backend: QuicBackend> RecvConnectionHandle<'a, Backend> {
    pub fn id(&'a self) -> ScopedId<'a> {
        self.id
    }

    pub fn state(&'a mut self) -> &'a mut Backend::ConnectionState {
        self.backend
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
struct RecvConnectionsQueryData<'w, Backend: QuicBackend> {
    shared: &'w mut ConnectionShared,
    state: &'w mut Connection<Backend::ConnectionState>,
    messages: &'w mut PeerMessages<Incoming>,
}