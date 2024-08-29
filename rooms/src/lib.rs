#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

use std::collections::BTreeSet;
use bevy_app::prelude::*;
use bevy_ecs::{prelude::*, system::EntityCommands, world::Command};
use bevy_stardust::prelude::*;
use smallvec::SmallVec;

#[cfg(feature="reflect")]
use bevy_reflect::prelude::*;

/// Adds support for rooms.
pub struct RoomsPlugin;

impl Plugin for RoomsPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature="reflect")]
        app.register_type::<Room>();

        // Observers
        app.observe(peer_comp_removed_observer);
        app.observe(room_comp_removed_observer);
    }
}

fn peer_comp_removed_observer(
    trigger: Trigger<OnRemove, Peer>,
    mut memberships: Query<&mut Memberships>,
    mut rooms: Query<&mut Room>,
) {

}

fn room_comp_removed_observer(
    trigger: Trigger<OnRemove, Room>,
    mut memberships: Query<&mut Memberships>,
    mut rooms: Query<&mut Room>,
) {

}

#[derive(Debug, Component)]
struct Memberships {
    incoming: SmallVec<[Entity; 3]>,
    outgoing: SmallVec<[Entity; 3]>,
}

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
    /// Creates a direct membership with `room`, using [`Join`].
    fn join(&mut self, room: Entity) -> &mut Self;

    /// Removes a direct membership with `room` if one exists, using [`Leave`].
    fn leave(&mut self, room: Entity) -> &mut Self;
}

impl RoomCommands for EntityCommands<'_> {
    fn join(&mut self, room: Entity) -> &mut Self {
        let peer = self.id();

        self.commands().add(Join {
            peer,
            room,
        });

        return self;
    }

    fn leave(&mut self, room: Entity) -> &mut Self {
        let peer = self.id();

        self.commands().add(Leave {
            peer,
            room,
        });

        return self;
    }
}

impl RoomCommands for EntityWorldMut<'_> {
    fn join(&mut self, room: Entity) -> &mut Self {
        let peer = self.id();

        let command = Join {
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

        let command = Leave {
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
pub struct Join {
    /// The peer that is to become a member of the room.
    /// May also be a room itself.
    pub peer: Entity,

    /// The room that the peer is to become a member of.
    pub room: Entity,
}

impl Command for Join {
    #[inline]
    fn apply(self, world: &mut World) {
        todo!()
    }
}

/// A command to remove a direct membership from a [`Peer`] (or `Room`) from a [`Room`].
#[derive(Debug, Clone)]
pub struct Leave {
    /// The peer that is to have its membership with the room removed.
    /// May also be a room itself.
    pub peer: Entity,

    /// The room that the membership is to removed from.
    pub room: Entity,
}

impl Command for Leave {
    #[inline]
    fn apply(self, world: &mut World) {
        todo!()
    }
}

fn must_check_for_cycle(
    query: Query<&Memberships>,
    parent: Entity,
    child: Entity,
) -> bool {
    debug_assert_ne!(parent, child);

    let (has_parents, has_children) = match query.get(parent) {
        Ok(val) => (
            val.incoming.len() == 0,
            val.outgoing.len() == 0,
        ),

        // With no Memberships component, the entity has no parents or children
        Err(_) => (false, false),
    };

    // If the node has no parents or no children, there cannot be a cycle
    return !has_parents || !has_children
}

struct Dfs {
    stack: Vec<Entity>,
    discovered: Vec<Entity>,
}

impl Dfs {
    fn new(start: Entity) -> Self {
        let mut stack = Vec::with_capacity(4);
        stack.push(start);

        Self {
            stack,
            discovered: Vec::with_capacity(16),
        }
    }

    fn reset(&mut self, from: Entity) {
        self.stack.clear();
        self.discovered.clear();
        self.stack.push(from);
    }

    fn next<F, I>(&mut self, mut func: F)
    where
        F: FnMut(Entity) -> I,
        I: Iterator<Item = Entity>,
    {
        // Repeatedly pop from the stack
        // This loop ends only when we've run out of nodes
        while let Some(node) = self.stack.pop() {
            // Check that we haven't already discovered this node
            if self.discovered.contains(&node) { continue }
            self.discovered.push(node);

            // Add newly discovered nodes to the stack
            for next in func(node) {
                if self.discovered.contains(&next) { continue }
                self.stack.push(next);
            }
        }
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