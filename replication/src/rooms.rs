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

fn member_relation_insert_observer(
    trigger: Trigger<SetEvent<Member>>,
    peers: Query<&ReplicationPeer>,
    mut rooms: Query<(&mut ReplicationRoom, Relations<Member>)>,
) {
    let host = trigger.entity();
    let target = trigger.event().target;

    if !peers.contains(host) {
        warn!("{host} is not a replication peer but was made the host of a member relation");
        return;
    }

    if !rooms.contains(target) {
        warn!("Replication peer {host} was made a member of a non-room entity {target}");
        return;
    }

    match rooms.contains(host) {
        true => {
            let mut discovered = BTreeSet::new();
            discovered.insert(host);

            // If the relation is a target from one room to another,
            // the target gains all the members from the first room
            rooms.traverse_mut::<Member>([target]).for_each(|room, _| {
                discovered.extend(&room.member_cache);
                room.member_cache.extend(&discovered);
                room.member_cache.insert(host);
            });
        },

        false => {
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