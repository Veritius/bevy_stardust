use std::marker::PhantomData;
use bevy::{ecs::{query::QueryEntityError, system::SystemParam}, prelude::*};
use bevy_stardust::prelude::*;
use crate::prelude::*;
use super::UseReplicationScope;

/// Utilities for reading global replication scope.
#[derive(SystemParam)]
pub struct ReplicationScopeHelpers<'w, 's> {
    enabled: Res<'w, State<UseReplicationScope>>,
    peers: Query<'w, 's, Entity, (With<NetworkPeer>, With<ReplicationPeer>)>,
    groups: Query<'w, 's, (Entity, &'static NetworkRoom, &'static NetworkGroup)>,
    ph: PhantomData<&'s ()>,
}

impl<'w, 's> ReplicationScopeHelpers<'w, 's> {
    /// Returns `true` if rooms and replication scoping is enabled.
    #[inline]
    pub fn rooms_enabled(&self) -> bool {
        self.enabled.get().is_enabled()
    }

    /// Returns `true` if the peer is in `room`.
    pub fn peer_in_room(&self, peer: Entity, room: Entity) -> Result<bool, QueryEntityError> {
        if self.peers.get(peer).is_err() { return Err(QueryEntityError::NoSuchEntity(peer)); }
        let (_, _, group) = self.groups.get(room)?;
        return Ok(group.contains(peer));
    }
}

/// Utilities for reading entity replication scope data.
#[derive(SystemParam)]
pub struct EntityReplicationScope<'w, 's> {
    general: ReplicationScopeHelpers<'w, 's>,
    entities: Query<'w, 's, (Entity, &'static ReplicateEntity)>,
}

impl<'w, 's> EntityReplicationScope<'w, 's> {
    /// Returns `true` if `entity` is replicated to `peer`.
    pub fn is_entity_replicated(&self, entity: Entity, peer: Entity) -> Result<bool, QueryEntityError> {
        // Entities are always replicated if scope is disabled.
        if !self.general.rooms_enabled() { return Ok(true); }

        todo!();
    }
}

/// Utilities for reading component replication scope data.
#[derive(SystemParam)]
pub struct ComponentReplicationScope<'w, 's, C: Component> {
    general: ReplicationScopeHelpers<'w, 's>,
    entities: EntityReplicationScope<'w, 's>,
    components: Query<'w, 's, &'static NetworkRoomMembership<C>>,
}

/// Utilities for reading resource replication scope data.
#[derive(SystemParam)]
pub struct ResourceReplicationScope<'w, 's, R: Resource> {
    general: ReplicationScopeHelpers<'w, 's>,
    membership: Res<'w, NetworkRoomMembership<R>>,
}

/// Utilities for reading event replication scope data.
#[derive(SystemParam)]
pub struct EventReplicationScope<'w, 's, E: Event> {
    general: ReplicationScopeHelpers<'w, 's>,
    membership: Res<'w, EventMemberships<E>>,
}

impl<'w, 's, E: Event> EventReplicationScope<'w, 's, E> {

}