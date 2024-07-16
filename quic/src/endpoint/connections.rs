use std::net::SocketAddr;
use bevy::prelude::Entity;
use crate::bimap::BiHashMap;

pub(crate) struct EndpointConnections {
    inner: BiHashMap<Entity, SocketAddr>,
}

impl EndpointConnections {
    pub fn new() -> Self {
        Self { inner: BiHashMap::new() }
    }

    /// SAFETY: An individual `id` can only be associated with one endpoint.
    pub unsafe fn register(&mut self, id: Entity, address: SocketAddr) {
        self.inner.insert(id, address);
    }

    pub fn deregister(&mut self, id: Entity) {
        self.inner.remove_left(&id);
    }

    pub fn get_address(&self, id: Entity) -> Option<SocketAddr> {
        self.inner.get_left(&id).cloned()
    }

    pub fn get_entity(&self, address: SocketAddr) -> Option<Entity> {
        self.inner.get_right(&address).cloned()
    }

    pub fn iter(&self) -> impl Iterator<Item = Entity> + '_ {
        self.inner.iter_left().cloned()
    }

    pub fn iter_owned(&self) -> impl Iterator<Item = Entity> {
        self.iter().collect::<Vec<_>>().into_iter()
    }
}