use std::sync::atomic::{AtomicU32, Ordering as AtomicOrdering};
use bevy::{prelude::*, utils::HashMap};

#[derive(Component)]
pub(crate) struct NetworkEntityIds {
    nte: HashMap<NetworkEntityId, Entity>,
    etn: HashMap<Entity, NetworkEntityId>,
    latest: AtomicU32,
}

impl NetworkEntityIds {
    pub fn id(&self) -> NetworkEntityId {
        NetworkEntityId(self.latest.fetch_add(1, AtomicOrdering::Relaxed))
    }

    pub fn add_pair(&mut self, eid: Entity, nid: NetworkEntityId) {
        self.nte.insert(nid, eid);
        self.etn.insert(eid, nid);
    }

    pub fn add_eid(&mut self, eid: Entity) -> NetworkEntityId {
        let nid = self.id();
        self.add_pair(eid, nid);
        return nid;
    }

    pub fn get_ent_id(&self, nid: NetworkEntityId) -> Option<Entity> {
        self.nte.get(&nid).copied()
    }

    pub fn get_net_id(&self, eid: Entity) -> Option<NetworkEntityId> {
        self.etn.get(&eid).copied()
    }

    pub fn remove_net_id(&mut self, id: NetworkEntityId) -> Option<Entity> {
        if let Some(ent) = self.nte.remove(&id) {
            self.etn.remove(&ent);
            return Some(ent);
        } else {
            return None;
        }
    }

    pub fn remove_ent_id(&mut self, id: Entity) -> Option<NetworkEntityId> {
        if let Some(nid) = self.etn.remove(&id) {
            self.nte.remove(&nid);
            return Some(nid);
        } else {
            return None;
        }
    }
}

/// Opaque entity ID relevant only to the entity that it originated from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct NetworkEntityId(u32);