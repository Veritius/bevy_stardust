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
    mut rooms: Query<&mut ReplicationRoom>,
    peers: Query<&ReplicationPeer>,
) {

}

fn member_relation_remove_observer(
    trigger: Trigger<UnsetEvent<Member>>,
    mut rooms: Query<&mut ReplicationRoom>,
    peers: Query<&ReplicationPeer>,
) {

}