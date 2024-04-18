use bevy::{ecs::{query::QueryEntityError, system::SystemParam}, prelude::*};
use bevy_stardust::prelude::*;
use crate::prelude::*;
use super::RoomsEnabled;

/// Utilities relating to replication scope, such as determining if an entity is in scope to a peer.
#[derive(SystemParam)]
pub struct ReplicationScope<'w, 's> {
    enabled: Option<Res<'w, RoomsEnabled>>,
    peers: Query<'w, 's, Entity, (With<NetworkPeer>, With<ReplicationPeer>)>,
    groups: Query<'w, 's, (&'static NetworkRoom, &'static NetworkGroup)>,
    entities: Query<'w, 's, &'static ReplicateEntity>,
}

impl<'w, 's> ReplicationScope<'w, 's> {
    /// Returns `true` if rooms and replication scoping is enabled.
    #[inline]
    pub fn rooms_enabled(&self) -> bool {
        self.enabled.is_some()
    }

    /// Returns `true` if the peer is in `room`.
    pub fn peer_in_room(&self, peer: Entity, room: Entity) -> Result<bool, QueryEntityError> {
        if self.peers.get(peer).is_err() { return Err(QueryEntityError::NoSuchEntity(peer)); }
        let (_, group) = self.groups.get(room)?;
        return Ok(group.contains(peer));
    }

    /// Returns `true` if `entity` is replicated to `peer`.
    pub fn is_entity_replicated(&self, entity: Entity, peer: Entity) -> Result<bool, QueryEntityError> {
        // Entities are always replicated if scope is disabled.
        if !self.rooms_enabled() { return Ok(true); }

        todo!();
    }
}