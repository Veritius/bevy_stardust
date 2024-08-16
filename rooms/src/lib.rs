#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

use std::collections::BTreeSet;
use bevy::prelude::*;
use bevy_stardust::prelude::*;
use aery::prelude::*;

/// Adds support for rooms.
pub struct RoomsPlugin;

impl Plugin for RoomsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Room>();
        app.register_relation::<Member>();

        app.observe(peer_component_insert_observer);
        app.observe(peer_component_remove_observer);
        app.observe(room_component_insert_observer);
        app.observe(room_component_remove_observer);
        app.observe(member_relation_insert_observer);
        app.observe(member_relation_remove_observer);
    }
}

/// An entity relation that makes a peer a member of a room.
#[derive(Relation)]
#[aery(Poly)]
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
    #[inline]
    pub fn contains(&self, peer: Entity) -> bool {
        self.cache.contains(&peer)
    }

    /// Returns an iterator over all the members of the room.
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

fn peer_component_insert_observer(
    trigger: Trigger<OnAdd, Peer>,
    mut commands: Commands,
    mut rooms: Query<(&mut Room, Relations<Member>)>,
) {
    todo!()
}

fn peer_component_remove_observer(
    trigger: Trigger<OnRemove, Peer>,
    mut commands: Commands,
    mut rooms: Query<(&mut Room, Relations<Member>)>,
) {
    todo!()
}

fn room_component_insert_observer(
    trigger: Trigger<OnAdd, Room>,
    mut commands: Commands,
    mut rooms: Query<(&mut Room, Relations<Member>)>,
) {
    todo!()
}

fn room_component_remove_observer(
    trigger: Trigger<OnAdd, Room>,
    mut commands: Commands,
    mut rooms: Query<(&mut Room, Relations<Member>)>,
) {
    todo!()
}

fn member_relation_insert_observer(
    trigger: Trigger<SetEvent<Member>>,
    mut commands: Commands,
    mut rooms: Query<(&mut Room, Relations<Member>)>,
) {
    todo!()
}

fn member_relation_remove_observer(
    trigger: Trigger<UnsetEvent<Member>>,
    mut commands: Commands,
    peers: Query<(), With<Peer>>,
    mut rooms: Query<((Entity, &mut Room), Relations<Member>)>,
) {
    let host = trigger.entity();
    let target = trigger.event().target;

    // Entities without Room cannot be targeted
    if !rooms.contains(target) { return }

    match rooms.contains(host) {
        true => {
            // Copy the cache into its own set so it can be iterated over
            // This is necessary since we need mutable access to the query later
            let ((_, room), _) = rooms.get(host).unwrap();
            let set = room.cache.iter().cloned().collect::<Vec<_>>();

            // Add members of the first room to all descendants
            rooms.traverse_mut::<Member>([target]).for_each(|(id, room), _| {
                for peer in set.iter().cloned() {
                    room.cache.insert(peer);
                    commands.trigger_targets(JoinedRoom { peer }, *id);
                }
            });
        },

        false => {
            // If it's not a room and not a peer, ignore it
            if !peers.contains(host) { return }

            // Add the ID of the peer to all descendants
            rooms.traverse_mut::<Member>([target]).for_each(|(id, room), _| {
                room.cache.insert(host);
                commands.trigger_targets(JoinedRoom { peer: host }, *id);
            });
        },
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
        world.observe(member_relation_insert_observer);

        let room_a = room(&mut world);
        let room_b = room(&mut world);

        let peer_a = peer(&mut world);
        let peer_b = peer(&mut world);
        let peer_c = peer(&mut world);

        // The room cache should start empty
        assert_eq!(room_cache(&world, room_a).len(), 0);

        world.entity_mut(peer_a).set::<Member>(room_a);
        assert!(room_cache(&world, room_a).contains(&peer_a));

        world.entity_mut(peer_b).set::<Member>(room_a);
        assert!(room_cache(&world, room_a).contains(&peer_a));
        assert!(room_cache(&world, room_a).contains(&peer_b));

        world.entity_mut(peer_c).set::<Member>(room_b);
        assert!(room_cache(&world, room_a).contains(&peer_a));
        assert!(room_cache(&world, room_a).contains(&peer_b));
        assert!(room_cache(&world, room_a).contains(&peer_c).not());
        assert!(room_cache(&world, room_b).contains(&peer_c));

        world.entity_mut(room_b).set::<Member>(room_a);
        assert!(room_cache(&world, room_a).contains(&peer_a));
        assert!(room_cache(&world, room_a).contains(&peer_b));
        assert!(room_cache(&world, room_a).contains(&peer_c));
        assert!(room_cache(&world, room_b).contains(&peer_a).not());
        assert!(room_cache(&world, room_b).contains(&peer_b).not());
        assert!(room_cache(&world, room_b).contains(&peer_c));
    }
}