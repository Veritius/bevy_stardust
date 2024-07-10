use std::net::{SocketAddr, UdpSocket};
use anyhow::Result;
use bevy::{prelude::*, utils::HashMap};
use crate::{Credentials, TrustAnchors};

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
    socket: UdpSocket,

    #[reflect(ignore)]
    ent_to_addr: HashMap<Entity, SocketAddr>,

    #[reflect(ignore)]
    addr_to_ent: HashMap<SocketAddr, Entity>,

    #[reflect(ignore)]
    #[cfg(feature="quiche")]
    pub(crate) quiche_config: quiche::Config,
}

impl Endpoint {
    pub(crate) fn socket(&self) -> &UdpSocket {
        &self.socket
    }

    /// SAFETY: An individual `id` can only be associated with one endpoint.
    pub(crate) unsafe fn insert_connection(&mut self, id: Entity, address: SocketAddr) {
        self.ent_to_addr.insert(id, address);
        self.addr_to_ent.insert(address, id);
    }

    pub(crate) fn remove_connection(&mut self, id: Entity) {
        if let Some(addr) = self.ent_to_addr.remove(&id) {
            self.addr_to_ent.remove(&addr);
        }
    }

    pub(crate) fn iterate_connections(&self) -> impl Iterator<Item = Entity> + '_ {
        self.ent_to_addr.keys().cloned()
    }

    pub(crate) fn iterate_connections_owned(&self) -> impl Iterator<Item = Entity> {
        self.iterate_connections().collect::<Vec<_>>().into_iter()
    }

    pub(crate) fn ent_to_addr(&mut self, id: Entity) -> Option<SocketAddr> {
        self.ent_to_addr.get(&id).cloned()
    }

    pub(crate) fn addr_to_ent(&mut self, id: SocketAddr) -> Option<Entity> {
        self.addr_to_ent.get(&id).cloned()
    }
}

impl Endpoint {
    /// Returns an [`EndpointBuilder`] which can be used to create a new `Endpoint`.
    #[inline]
    pub fn builder() -> EndpointBuilder {
        EndpointBuilder::new()
    }

    /// Returns the local address this endpoint is bound to.
    pub fn local_addr(&self) -> SocketAddr {
        self.socket.local_addr().unwrap()
    }
}

pub struct EndpointBuilder {
    trust_anchors: Option<TrustAnchors>,
    credentials: Option<Credentials>,
}

impl EndpointBuilder {
    pub fn new() -> Self {
        Self {
            trust_anchors: None,
            credentials: None,
        }
    }

    pub fn with_trust_anchors(mut self, trust_anchors: TrustAnchors) -> Self {
        self.trust_anchors = Some(trust_anchors);
        return self;
    }

    pub fn with_credentials(mut self, credentials: Credentials) -> Self {
        self.credentials = Some(credentials);
        return self;
    }

    pub fn build(self) -> Result<Endpoint> {
        todo!()
    }
}