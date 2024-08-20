#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

use std::collections::BTreeSet;
use bevy_app::prelude::*;
use bevy_ecs::{prelude::*, system::EntityCommands, world::Command};
use bevy_stardust::prelude::*;

#[cfg(feature="reflect")]
use bevy_reflect::prelude::*;

/// Adds support for rooms.
pub struct RoomsPlugin;

impl Plugin for RoomsPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature="reflect")]
        app.register_type::<Room>();
    }
}

/// An entity relation that makes a peer a member of a room.
pub struct Member;

/// A collection of peers.
/// 
/// Rooms are defined by their [members](crate::Member).
#[derive(Debug, Component, Reflect)]
#[reflect(Default, Component)]
pub struct Room {
    #[reflect(ignore)]
    cache: BTreeSet<Entity>,
}

impl Room {
    /// Creates a new `Room` component.
    pub fn new() -> Self {
        Self {
            cache: BTreeSet::new(),
        }
    }

    /// Returns `true` if `peer` is considered a member of the room.
    /// 
    /// This is meaningless unless the component is part of the `World`.
    #[inline]
    pub fn contains(&self, peer: Entity) -> bool {
        self.cache.contains(&peer)
    }

    /// Returns an iterator over all the members of the room.
    /// 
    /// This is meaningless unless the component is part of the `World`.
    pub fn iter(&self) -> RoomIter {
        RoomIter {
            iter: self.cache.iter(),
        }
    }
}

impl Default for Room {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// An iterator over members of a [`Room`].
/// 
/// The iterator is in sorted order based on the `Ord` implementation of `Entity`.
#[derive(Clone)]
pub struct RoomIter<'a> {
    iter: std::collections::btree_set::Iter<'a, Entity>,
}

impl<'a> Iterator for RoomIter<'a> {
    type Item = Entity;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().copied()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

/// An observer trigger raised when a peer joins a [`Room`].
#[derive(Event)]
pub struct JoinedRoom {
    /// The ID of the peer that left the room.
    pub peer: Entity,
}

/// An observer trigger raised when a peer leaves a [`Room`].
#[derive(Event)]
pub struct LeftRoom {
    /// The ID of the peer that joined the room.
    pub peer: Entity,
}

/// An extension API for working with rooms.
pub trait RoomCommands {
    /// Creates a direct membership with `room`, using [`JoinRoom`].
    fn join(&mut self, room: Entity) -> &mut Self;

    /// Removes a direct membership with `room` if one exists, using [`LeaveRoom`].
    fn leave(&mut self, room: Entity) -> &mut Self;
}

impl RoomCommands for EntityCommands<'_> {
    fn join(&mut self, room: Entity) -> &mut Self {
        let peer = self.id();

        self.commands().add(JoinRoom {
            peer,
            room,
        });

        return self;
    }

    fn leave(&mut self, room: Entity) -> &mut Self {
        let peer = self.id();

        self.commands().add(LeaveRoom {
            peer,
            room,
        });

        return self;
    }
}

impl RoomCommands for EntityWorldMut<'_> {
    fn join(&mut self, room: Entity) -> &mut Self {
        let peer = self.id();

        let command = JoinRoom {
            peer,
            room,
        };

        self.world_scope(|world| {
            Command::apply(command, world);
        });

        self.update_location();

        return self;
    }

    fn leave(&mut self, room: Entity) -> &mut Self {
        let peer = self.id();

        let command = LeaveRoom {
            peer,
            room,
        };

        self.world_scope(|world| {
            Command::apply(command, world);
        });

        self.update_location();

        return self;
    }
}

/// A command to add a direct membership from a [`Peer`] (or `Room`) to a [`Room`].
#[derive(Debug, Clone)]
pub struct JoinRoom {
    /// The peer that is to become a member of the room.
    /// May also be a room itself.
    pub peer: Entity,

    /// The room that the peer is to become a member of.
    pub room: Entity,
}

impl Command for JoinRoom {
    #[inline]
    fn apply(self, world: &mut World) {
        todo!()
    }
}

/// A command to remove a direct membership from a [`Peer`] (or `Room`) from a [`Room`].
#[derive(Debug, Clone)]
pub struct LeaveRoom {
    /// The peer that is to have its membership with the room removed.
    /// May also be a room itself.
    pub peer: Entity,

    /// The room that the membership is to removed from.
    pub room: Entity,
}

impl Command for LeaveRoom {
    #[inline]
    fn apply(self, world: &mut World) {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Not;
    use super::*;

    fn room(world: &mut World) -> Entity {
        world.spawn(Room::default()).id()
    }

    fn peer(world: &mut World) -> Entity {
        world.spawn(Peer::new()).id()
    }

    fn room_cache(world: &World, id: Entity) -> &BTreeSet<Entity> {
        &world.get::<Room>(id).unwrap().cache
    }

    #[test]
    fn member_cache_addition() {
        let mut world = World::new();

        let room_a = room(&mut world);
        let room_b = room(&mut world);

        let peer_a = peer(&mut world);
        let peer_b = peer(&mut world);
        let peer_c = peer(&mut world);

        // The room cache should start empty
        assert_eq!(room_cache(&world, room_a).len(), 0);

        world.entity_mut(peer_a).join(room_a);
        assert!(room_cache(&world, room_a).contains(&peer_a));

        world.entity_mut(peer_b).join(room_a);
        assert!(room_cache(&world, room_a).contains(&peer_a));
        assert!(room_cache(&world, room_a).contains(&peer_b));

        world.entity_mut(peer_c).join(room_b);
        assert!(room_cache(&world, room_a).contains(&peer_a));
        assert!(room_cache(&world, room_a).contains(&peer_b));
        assert!(room_cache(&world, room_a).contains(&peer_c).not());
        assert!(room_cache(&world, room_b).contains(&peer_c));

        world.entity_mut(room_b).join(room_a);
        assert!(room_cache(&world, room_a).contains(&peer_a));
        assert!(room_cache(&world, room_a).contains(&peer_b));
        assert!(room_cache(&world, room_a).contains(&peer_c));
        assert!(room_cache(&world, room_b).contains(&peer_a).not());
        assert!(room_cache(&world, room_b).contains(&peer_b).not());
        assert!(room_cache(&world, room_b).contains(&peer_c));
    }
}