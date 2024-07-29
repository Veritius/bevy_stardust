use std::{collections::BTreeMap, net::{SocketAddr, UdpSocket}};
use bevy::prelude::*;
use quinn_proto::ConnectionHandle;
use crate::connections::token::ConnectionOwnershipToken;

/// A QUIC endpoint using `quinn_proto`.
#[derive(Component)]
pub struct Endpoint {
    /// The size of a buffer allocated to receive datagrams.
    /// Higher values allow remote peers to send data more efficiently.
    /// 
    /// The amount of space allocated, in bytes, is equal to the value of this field.
    /// 
    /// If this is set to below `1280`, QUIC packets may be cut off and become unreadable.
    /// Most operating systems also do not buffer UDP datagrams bigger than `65535` bytes,
    /// so setting this field that high may simply waste memory, depending on the operating system.
    pub recv_buf_size: u16,

    pub(crate) socket: UdpSocket,

    pub(crate) quinn: quinn_proto::Endpoint,

    connections: BTreeMap<ConnectionHandle, ConnectionOwnershipToken>,
}

impl Endpoint {
    /// Returns the local address of the [`Endpoint`].
    pub fn local_addr(&self) -> SocketAddr {
        self.socket.local_addr().unwrap()
    }
}

#[cfg(debug_assertions)]
pub(crate) fn safety_check_system(
    endpoints: Query<&Endpoint>,
) {
    use std::collections::BTreeSet;

    let mut tokens = BTreeSet::new();

    for endpoint in &endpoints {
        for connection in endpoint.connections.values() {
            assert!(!tokens.insert(connection.inner()), 
                "Two ConnectionOwnershipTokens existed simultaneously. This is undefined behavior!");
        }
    }
}