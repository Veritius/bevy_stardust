use std::net::UdpSocket;
use bevy::{prelude::*, utils::HashSet};

/// A QUIC endpoint, corresponding to a single UDP socket.
/// 
/// All [connections](crate::Connection) 'belong' to an Endpoint, which they use for I/O.
#[derive(Component, Reflect)]
#[reflect(from_reflect=false, Component)]
pub struct Endpoint {
    /// The amount of space that is allocated to receive packets.
    /// This must be at least `1280`, the minimum packet size imposed by the QUIC standard.
    /// Setting this above `65535` is pointless, as that is the largest packet size in most operating systems.
    #[reflect(@1280..65535)]
    pub recv_mtu: usize,

    #[reflect(ignore)]
    connections: HashSet<Entity>,

    #[reflect(ignore)]
    socket: UdpSocket,
}

impl Endpoint {
    /// SAFETY: An individual `id` can only be associated with one endpoint.
    pub(crate) unsafe fn insert_connection(&mut self, id: Entity) {
        self.connections.insert(id);
    }

    pub(crate) fn remove_connection(&mut self, id: Entity) {
        self.connections.remove(&id);
    }
}