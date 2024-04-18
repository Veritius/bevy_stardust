use std::sync::atomic::{AtomicU32, Ordering as AtomicOrdering};
use bevy::{prelude::*, utils::HashMap};

use super::Side;

#[derive(Component)]
pub(crate) struct NetworkEntityIds {
    nte: HashMap<NetworkEntityId, Entity>,
    etn: HashMap<Entity, NetworkEntityId>,
    side: Side,
    latest: AtomicU32,
}

impl NetworkEntityIds {
    pub fn new(side: Side) -> Self {
        Self {
            nte: HashMap::default(),
            etn: HashMap::default(),
            side,
            latest: AtomicU32::new(0),
        }
    }

    pub fn id(&self) -> NetworkEntityId {
        let int = self.latest.fetch_add(1, AtomicOrdering::Relaxed);
        NetworkEntityId::new(self.side, int).expect("Exceeded networked entity id limit")
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

/// Opaque entity ID relevant only to connection it originated from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Reflect)]
pub(crate) struct NetworkEntityId(u32);

impl NetworkEntityId {
    pub const MIN: u32 = u32::MIN;
    pub const MAX: u32 = (1 << 31) - 1;

    const FLAG: u32 = 1 << 31;

    fn new(side: Side, value: u32) -> Result<Self, ()> {
        // Check for the numerical range limit
        if Self::left_high(value) { return Err(()); }

        // Create inner value
        let mut val = value;
        if side == Side::Client { val |= Self::FLAG; }
        return Ok(Self(val))
    }

    #[inline]
    fn left_high(value: u32) -> bool {
        value & Self::FLAG > 0
    }

    pub fn side(&self) -> Side {
        match Self::left_high(self.0) {
            true => Side::Client,
            false => Side::Server,
        }
    }

    #[inline]
    pub fn to_bytes(&self) -> [u8; 4] {
        self.0.to_be_bytes()
    }

    #[inline]
    pub fn from_bytes(bytes: [u8; 4]) -> Self {
        Self(u32::from_be_bytes(bytes))
    }
}

impl From<NetworkEntityId> for u32 {
    fn from(value: NetworkEntityId) -> Self {
        // Always disable the is-left flag.
        value.0 & !NetworkEntityId::FLAG
    }
}

impl From<[u8; 4]> for NetworkEntityId {
    #[inline]
    fn from(value: [u8; 4]) -> Self {
        Self::from_bytes(value)
    }
}

impl From<NetworkEntityId> for [u8; 4] {
    #[inline]
    fn from(value: NetworkEntityId) -> Self {
        value.to_bytes()
    }
}

/// Storage for the IDs that identify this entity, per peer.
#[derive(Default)]
pub(crate) struct AssociatedNetworkIds(HashMap<Entity, NetworkEntityId>);

impl AssociatedNetworkIds {
    #[inline]
    pub fn insert(&mut self, peer: Entity, id: NetworkEntityId) {
        self.0.insert(peer, id);
    }

    #[inline]
    pub fn remove(&mut self, peer: Entity) {
        self.0.remove(&peer);
    }

    #[inline]
    pub fn get(&self, peer: Entity) -> Option<NetworkEntityId> {
        self.0.get(&peer).cloned()
    }

    #[inline]
    pub fn all(&self) -> impl Iterator<Item = (Entity, NetworkEntityId)> + '_ {
        self.0.iter().map(|(k,v)| { (k.clone(), v.clone()) })
    }
}