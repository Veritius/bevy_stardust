use std::{net::{SocketAddr, UdpSocket}, collections::BTreeMap};
use bevy::prelude::*;
use crate::shared::user::NetworkUserId;

/// Creates new, unique `NetworkUserId`s.
#[derive(Resource)]
pub(crate) struct NetworkIdAssigner(u32);

impl NetworkIdAssigner {
    pub fn generate(&mut self) -> NetworkUserId {
        if self.0 == u32::MAX { panic!("Ran out of IDs to assign to users"); }
        let id = NetworkUserId(self.0);
        self.0 += 1;
        id
    }
}

impl Default for NetworkIdAssigner {
    fn default() -> Self {
        Self(1) // The server always has 0 as its id
    }
}

#[derive(Resource)]
pub(super) struct SocketMemory {
    pub entities: BTreeMap<Entity, NetworkUserId>,
    pub sockets: BTreeMap<NetworkUserId, UdpSocket>,
}

/// Represents a connected client as an entity.
/// 
/// Despawning an entity with this component will disconnect the client.
#[derive(Debug, Component)]
pub struct Client {
    pub(super) id: NetworkUserId,
}

impl Client {
    /// Returns this client's NetworkUserId.
    pub fn id(&self) -> NetworkUserId {
        self.id
    }
}

pub(super) fn client_comp_despawn_disconnection_system(
    mut removals: RemovedComponents<Client>,
    mut memory: ResMut<SocketMemory>,
) {
    for removal in removals.iter() {
        let id: NetworkUserId;

        // Sanity checks
        if let Some(net_id) = memory.entities.get(&removal) {
            if !memory.sockets.contains_key(net_id) {
                // This should never happen.
                panic!("Network user {:?} disconnected but didn't have an associated socket in the map", net_id)
            }
            id = net_id.clone();
        } else {
            // This should also never happen.
            panic!("The Client component was removed from an entity that wasn't in the removal memory")
        }

        // Remove from memory
        memory.sockets.remove(&id);
        memory.entities.remove(&removal);
        info!("User {:?} disconnected by component removal", id);
    }
}