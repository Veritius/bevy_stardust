use std::sync::atomic::{AtomicU32, Ordering as AtomicOrdering};
use bevy::{prelude::*, utils::HashMap};

#[derive(Component)]
pub(crate) struct NetworkEntityIds {
    ltr: HashMap<NetworkEntityId, Entity>,
    rtl: HashMap<Entity, NetworkEntityId>,
    latest: AtomicU32,
}

impl NetworkEntityIds {
    pub fn id(&self) -> NetworkEntityId {
        NetworkEntityId(self.latest.fetch_add(1, AtomicOrdering::Relaxed))
    }

    pub fn add(&mut self, id: NetworkEntityId, ent: Entity) {
        self.ltr.insert(id, ent);
        self.rtl.insert(ent, id);
    }
}

/// Opaque entity ID relevant only to the entity that it originated from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct NetworkEntityId(u32);