use bevy::{ecs::system::{EntityCommand, EntityCommands}, prelude::*};
use bevy_stardust::prelude::*;

/// Defines a 'network room' entity. This filters the entities that are replicated to each peer.
/// Entity rooms are a many-to-many relationship that are cheap to iterate over.
/// 
/// Peers considered members of the room (as per [`NetworkGroup`]) will have entities replicated to them.
/// Entities considered members of the group (as per this component) will be replicated to peer members.
#[derive(Debug, Component, Reflect)]
#[reflect(Debug, Component)]
pub struct NetworkRoom {
    /// See [`RoomFilterMode`]'s documentation.
    pub filter: RoomFilterMode,
    /// See [`RoomHierarchyMode`]'s documentation.
    pub hierarchy: RoomHierarchyMode,

    members: Vec<Entity>,
}

impl NetworkRoom {
    /// Returns `true` if `entity` is a member of the network room.
    pub fn contains_entity(&self, entity: Entity) {
        match self.members.binary_search(&entity) {
            Ok(_) => true,
            Err(_) => false,
        };
    }

    /// Returns all entities that are in the room's scope, in sorted order.
    pub fn members(&self) -> &[Entity] {
        &self.members
    }

    /// Add a membership.
    fn add(&mut self, room: Entity) {
        if let Err(index) = self.members.binary_search(&room) {
            self.members.insert(index, room);
        }
    }

    /// Remove a membership.
    fn remove(&mut self, room: Entity) {
        if let Ok(index) = self.members.binary_search(&room) {
            self.members.remove(index);
        }
    }
}

/// A bundle for a minimal network room.
#[derive(Bundle)]
#[allow(missing_docs)]
pub struct NetworkRoomBundle {
    pub room: NetworkRoom,
    pub group: NetworkGroup,
}

/// Defines how peers in the room should be filtered out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
#[reflect(Debug, PartialEq)]
pub enum RoomFilterMode {
    /// Only peers in the room will have entities replicated to them.
    Exclusive,

    /// Only peers outside of the room will have entities replicated to them.
    Inclusive,
}

/// Defines how items in the hierarchy should be treated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
#[reflect(Debug, PartialEq)]
pub enum RoomHierarchyMode {
    /// Only entities specifically defined in the room are considered in scope.
    Singular,
    /// Any and all child entities of in-scope entities will also be considered in scope.
    Recursive,
}

/// Stores membership data.
#[derive(Debug, Component, Reflect)]
#[reflect(Debug, Component)]
pub struct NetworkRoomMember(Vec<Entity>);

impl NetworkRoomMember {
    /// Returns `true` if a member of `room`.
    pub fn in_room(&self, room: Entity) -> bool {
        match self.0.binary_search(&room) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    /// Returns all rooms this entity is part of, in sorted order.
    pub fn rooms(&self) -> &[Entity] {
        &self.0
    }

    /// Add a membership.
    fn add(&mut self, room: Entity) {
        if let Err(index) = self.0.binary_search(&room) {
            self.0.insert(index, room);
        }
    }

    /// Remove a membership.
    fn remove(&mut self, room: Entity) {
        if let Ok(index) = self.0.binary_search(&room) {
            self.0.remove(index);
        }
    }
}

/// Adds an entity to a network room.
#[derive(Debug, Clone, Copy)]
pub struct AddToNetworkRoom {
    /// The room to add the entity to.
    pub room: Entity,
}

impl EntityCommand for AddToNetworkRoom {
    fn apply(self, id: Entity, world: &mut World) {
        let room = self.room;
        if id == room {
            error!("Tried to add a room as a member of itself: {id:?}");
            return;
        }

        let mut room_entity = world.entity_mut(room);
        if let Some(mut comp) = room_entity.get_mut::<NetworkRoom>() {
            comp.add(id);
        } else {
            error!("Tried to add to a room but target wasn't a room: {room:?}");
            return;
        }

        let mut member_entity = world.entity_mut(room);
        if let Some(mut comp) = member_entity.get_mut::<NetworkRoomMember>() {
            comp.add(room);
        } else {
            member_entity.insert(NetworkRoomMember(vec![room]));
        }
    }
}

/// Removes an entity from a network room.
#[derive(Debug, Clone, Copy)]
pub struct RemoveFromNetworkRoom {
    /// The room to remove the entity from.
    pub room: Entity,
}

impl EntityCommand for RemoveFromNetworkRoom {
    fn apply(self, id: Entity, world: &mut World) {
        let room = self.room;
        if id == room {
            error!("Tried to remove a room as a member of itself: {id:?}");
            return;
        }

        let mut room_entity = world.entity_mut(room);
        if let Some(mut comp) = room_entity.get_mut::<NetworkRoom>() {
            comp.remove(id);
        } else {
            return;
        }

        let mut member_entity = world.entity_mut(room);
        if let Some(mut comp) = member_entity.get_mut::<NetworkRoomMember>() {
            comp.remove(room);
        } else {
            unreachable!();
        }
    }
}

/// Extension trait for [`EntityCommands`] for network room related commands.
pub trait RoomEntityCommandExts: sealed::Sealed {
    /// Adds the entity to a network room (adds [`AddToNetworkRoom`])
    /// If the target is not a network room, nothing happens.
    fn add_to_room(&mut self, room: Entity) -> &mut Self;

    /// Removes the entity from a network room (adds [`RemoveFromNetworkRoom`])
    fn remove_from_room(&mut self, room: Entity) -> &mut Self;
}

impl RoomEntityCommandExts for EntityCommands<'_> {
    fn add_to_room(&mut self, room: Entity) -> &mut Self {
        self.add(AddToNetworkRoom { room });
        return self;
    }

    fn remove_from_room(&mut self, room: Entity) -> &mut Self {
        self.add(RemoveFromNetworkRoom { room });
        return self;
    }
}

mod sealed {
    use super::*;

    pub trait Sealed {}
    impl Sealed for EntityCommands<'_> {}
}