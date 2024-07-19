use bevy::{prelude::Entity, utils::EntityHashSet};

pub(crate) struct EndpointConnections {
    inner: EntityHashSet<Entity>,
}

impl EndpointConnections {
    pub(super) fn new() -> Self {
        Self { inner: EntityHashSet::default() }
    }

    /// SAFETY: An individual `id` can only be associated with one endpoint.
    pub(super) unsafe fn register(&mut self, id: Entity) {
        self.inner.insert(id);
    }

    pub(super) fn deregister(&mut self, id: Entity) {
        self.inner.remove(&id);
    }

    pub fn contains(&self, id: Entity) -> bool {
        self.inner.contains(&id)
    }

    pub fn iter(&self) -> impl Iterator<Item = Entity> + '_ {
        self.inner.iter().cloned()
    }
}