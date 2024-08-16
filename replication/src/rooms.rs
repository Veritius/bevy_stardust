//! Replication rooms.

use std::collections::BTreeSet;
use bevy::prelude::*;
use aery::prelude::*;
use crate::connections::ReplicationPeer;

/// Adds support for [replication rooms](ReplicationRoom).
pub struct RoomsPlugin;

impl Plugin for RoomsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ReplicationRoom>();

        // Various observers
        app.observe(member_relation_insert_observer);
        app.observe(member_relation_remove_observer);
    }
}

/// A [`Relation`] identifying a [`ReplicationPeer`] as a member of a [`ReplicationRoom`].
#[derive(Relation)]
#[aery(Poly)]
pub struct Member;

/// A replication room, allowing configuration to be applied to many peers at once.
#[derive(Debug, Component, Reflect)]
#[reflect(Component)]
pub struct ReplicationRoom {
    member_cache: BTreeSet<Entity>,
}

impl Default for ReplicationRoom {
    fn default() -> Self {
        Self {
            member_cache: BTreeSet::new(),
        }
    }
}

impl ReplicationRoom {
    /// Returns `true` if the room contains `peer`.
    #[inline]
    pub fn contains(&self, peer: Entity) -> bool {
        self.member_cache.contains(&peer)
    }
}

fn member_relation_insert_observer(
    trigger: Trigger<SetEvent<Member>>,
    peers: Query<&ReplicationPeer>,
    mut rooms: Query<(&mut ReplicationRoom, Relations<Member>)>,
) {
    let host = trigger.entity();
    let target = trigger.event().target;

    if !rooms.contains(target) {
        warn!("Replication peer {host} was made a member of a non-room entity {target}");
        return;
    }

    match rooms.contains(host) {
        true => {
            // Copy the cache into its own set so it can be iterated over
            // This is necessary since we need mutable access to the query later
            let (room, _) = rooms.get(host).unwrap();
            let set = room.member_cache.iter().cloned().collect::<Vec<_>>();

            // If the relation is a target from one room to another,
            // the target gains all the members from the first room
            rooms.traverse_mut::<Member>([target]).for_each(|room, _| {
                room.member_cache.extend(&set);
            });
        },

        false => {
            if !peers.contains(host) {
                warn!("{host} is not a replication peer but was made the host of a member relation");
                return;
            }

            // If the relation host is just a replication peer,
            // it's simply inserted into all descendants
            rooms.traverse_mut::<Member>([target]).for_each(|room, _| {
                room.member_cache.insert(host);
            });
        },
    }
}

fn member_relation_remove_observer(
    trigger: Trigger<UnsetEvent<Member>>,
    peers: Query<&ReplicationPeer>,
    mut rooms: Query<(&mut ReplicationRoom, Relations<Member>)>,
) {
    let host = trigger.entity();
    let target = trigger.event().target;

    if !peers.contains(host) { return; }
    if !rooms.contains(target) { return; }
}

#[cfg(test)]
mod tests {
    use std::ops::Not;
    use super::*;

    fn room(world: &mut World) -> Entity {
        world.spawn(ReplicationRoom::default()).id()
    }

    fn peer(world: &mut World) -> Entity {
        use crate::identifiers::Side;
        world.spawn(ReplicationPeer::new(Side::Left)).id()
    }

    fn room_cache(world: &World, id: Entity) -> &BTreeSet<Entity> {
        &world.get::<ReplicationRoom>(id).unwrap().member_cache
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