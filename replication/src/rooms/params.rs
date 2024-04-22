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

        // todo!();
        return Ok(true);
    }
}

/// Utilities for reading component replication scope data.
#[derive(SystemParam)]
pub struct ComponentReplicationScope<'w, 's, C: Component> {
    general: ReplicationScopeHelpers<'w, 's>,
    entities: EntityReplicationScope<'w, 's>,
    components: Query<'w, 's, &'static NetworkRoomMembership<C>>,
}

impl<'w, 's, C: Component> ComponentReplicationScope<'w, 's, C> {
    /// Returns whether the component `C` on `entity` is replicated to `peer`.
    pub fn is_component_replicated_to(&self, entity: Entity, peer: Entity) -> Result<bool, QueryEntityError> {
        // Components are not visible if their entity is not visible.
        if !self.entities.is_entity_replicated(entity, peer)? { return Ok(false); }

        // todo!()
        return Ok(true);
    }
}

/// Utilities for reading resource replication scope data.
#[derive(SystemParam)]
pub struct ResourceReplicationScope<'w, 's, R: Resource> {
    general: ReplicationScopeHelpers<'w, 's>,
    membership: Res<'w, NetworkRoomMembership<R>>,
}

impl<'w, 's, R: Resource> ResourceReplicationScope<'w, 's, R> {
    /// Returns whether the resource of type `R` is replicated to `peer`.
    pub fn is_resource_replicated(&self, peer: Entity) -> Result<bool, QueryEntityError> {
        if self.general.rooms_enabled() { return Ok(true); }

        // todo!()
        return Ok(true);
    }
}

/// Utilities for reading event replication scope data.
#[derive(SystemParam)]
pub struct EventReplicationScope<'w, 's, E: Event> {
    general: ReplicationScopeHelpers<'w, 's>,
    membership: Res<'w, EventMemberships<E>>,
}

impl<'w, 's, E: Event> EventReplicationScope<'w, 's, E> {
    /// Returns whether the event of type `E` is replicated to `peer`.
    pub fn is_event_replicated(&self, peer: Entity) -> Result<bool, QueryEntityError> {
        if self.general.rooms_enabled() { return Ok(true); }

        // todo!()
        return Ok(true);
    }
}