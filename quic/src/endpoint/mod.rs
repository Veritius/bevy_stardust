mod builder;
mod connections;

use std::net::{SocketAddr, UdpSocket};
use bevy::prelude::*;

pub use builder::{EndpointBuilder, Client, Server, Dual};

pub(crate) use connections::EndpointConnections;

#[allow(unused)] // various backends may or may not use these exports
pub(crate) use builder::{ReadyShared, HostShared, JoinShared, ClientReady, ServerReady, DualReady};

/// A QUIC endpoint, corresponding to a single UDP socket.
/// 
/// All [connections](crate::Connection) 'belong' to an Endpoint, which they use for I/O.
#[derive(Component, Reflect)]
#[reflect(from_reflect=false, Component)]
pub struct Endpoint {
    /// If `true`, the endpoint will listen for new, incoming connections.
    pub listening: bool,

    /// The amount of space that is allocated to transmitting UDP packets.
    /// This must be at least `1280`, the minimum packet size imposed by the QUIC standard.
    /// Setting this above `65535` is pointless, as that is the largest packet size in most operating systems.
    #[reflect(@1280..65535)]
    pub send_size: usize,

    /// The amount of space that is allocated to receiving UDP packets.
    /// This must be at least `1280`, the minimum packet size imposed by the QUIC standard.
    /// Setting this above `65535` is pointless, as that is the largest packet size in most operating systems.
    #[reflect(@1280..65535)]
    pub recv_size: usize,

    #[reflect(ignore)]
    pub(crate) socket: UdpSocket,

    #[reflect(ignore)]
    pub(crate) connections: EndpointConnections,

    #[reflect(ignore)]
    #[cfg(feature="quiche")]
    pub(crate) quiche_config: quiche::Config,
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
}