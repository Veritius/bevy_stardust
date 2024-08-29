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
    mut memberships: Query<&mut DirectMemberships>,
    mut rooms: Query<&mut Room>,
) {

}

fn room_comp_removed_observer(
    trigger: Trigger<OnRemove, Room>,
    mut memberships: Query<&mut DirectMemberships>,
    mut rooms: Query<&mut Room>,
) {

}

#[derive(Debug, Default, Component)]
struct DirectMemberships {
    incoming: SmallVec<[Entity; 3]>,
    outgoing: SmallVec<[Entity; 3]>,
}

impl DirectMemberships {
    fn with_incoming(mut self, id: Entity) -> Self {
        self.incoming.push(id);
        return self;
    }

    fn with_outgoing(mut self, id: Entity) -> Self {
        self.outgoing.push(id);
        return self;
    }
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
        // Check that the host and target are not the same entity
        if self.peer == self.room {
            #[cfg(feature="log")]
            bevy_log::debug!("Peer {} cannot be a member of itself", self.peer);

            return;
        }

        // Membership query + test that the host is not already a direct of the target
        let mut memberships = world.query::<&mut DirectMemberships>();
        if let Ok(membership) = memberships.get_manual(world, self.peer) {
            if membership.incoming.binary_search(&self.room).is_ok() {
                #[cfg(feature="log")]
                bevy_log::debug!("Peer {} was already a member of {}", self.peer, self.room);

                return;
            }
        }

        // Rooms query + check that the target is a room
        let mut rooms = world.query::<&mut Room>();
        if rooms.get_manual(world, self.room).is_err() {
            #[cfg(feature="log")]
            bevy_log::debug!("{} is not a room entity", self.room);

            return;
        }

        let mut dfs = DfsState::new(self.peer);

        // Preliminary check to see if adding a link between the host and target would cause a cycle
        // This is significantly cheaper than the in-depth check that traverses the graph
        if must_check_for_cycle(&world, memberships.as_readonly(), self.peer, self.room) {
            // In-depth check that traverses the graph to find a cycle
            if has_connecting_path(&world, memberships.as_readonly(), self.peer, self.room, &mut dfs) {
                #[cfg(feature="log")]
                bevy_log::warn!("Making {} a member of {} would cause a cycle", self.peer, self.room);

                return;
            }
        }

        match memberships.get_mut(world, self.peer) {
            Ok(mut memberships) => memberships.outgoing.push(self.room),
            Err(_) => match world.get_entity_mut(self.room) {
                Some(mut entity) => { entity.insert(DirectMemberships::default().with_outgoing(self.room)); },
                None => {
                    #[cfg(feature="log")]
                    bevy_log::debug!("{} was not spawned when join command ran. Was it despawned?", self.peer);

                    return;
                },
            },
        }

        match memberships.get_mut(world, self.room) {
            Ok(mut memberships) => memberships.incoming.push(self.peer),
            Err(_) => match world.get_entity_mut(self.room) {
                Some(mut entity) => { entity.insert(DirectMemberships::default().with_outgoing(self.peer)); },
                None => {
                    #[cfg(feature="log")]
                    bevy_log::debug!("{} was not spawned when join command ran. Was it despawned?", self.room);

                    return;
                },
            },
        }

        dfs.reset(self.peer);

        let func = |next| match memberships.get(world, self.peer) {
            Ok(memberships) => Some(memberships.incoming.iter().copied()),
            Err(_) => None,
        };

        while let Some(node) = dfs.next(func) {
            todo!()
        }

        #[cfg(feature="log")]
        bevy_log::trace!("Made {} a member of {}", self.peer, self.room);
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
    world: &World,
    query: &QueryState<&DirectMemberships>,
    parent: Entity,
    child: Entity,
) -> bool {
    debug_assert_ne!(parent, child);

    let (has_parents, has_children) = match query.get_manual(world, parent) {
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

fn has_connecting_path(
    world: &World,
    query: &QueryState<&DirectMemberships>,
    parent: Entity,
    child: Entity,
    dfs: &mut DfsState,
) -> bool {
    dfs.reset(parent);

    let mut func = |next| match query.get_manual(world, next) {
        Ok(memberships) => Some(memberships.outgoing.iter().copied()),
        Err(_) => None,
    };

    while let Some(node) = dfs.next(&mut func) {
        if child == node { return true }
    }

    return false;
}

struct DfsState {
    stack: Vec<Entity>,
    discovered: Vec<Entity>,
}

impl DfsState {
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

    fn next<F, I>(&mut self, mut func: F) -> Option<Entity>
    where
        F: FnMut(Entity) -> Option<I>,
        I: Iterator<Item = Entity>,
    {
        // Repeatedly pop from the stack
        // This loop ends only when we've run out of nodes
        while let Some(node) = self.stack.pop() {
            // Check that we haven't already discovered this node
            if self.discovered.contains(&node) { continue }
            self.discovered.push(node);

            // Add newly discovered nodes to the stack
            if let Some(iter) = func(node) {
                for next in iter {
                    if self.discovered.contains(&next) { continue }
                    self.stack.push(next);
                }
            }

            // Return the nodes
            return Some(node);
        }

        // Out of nodes
        return None;
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