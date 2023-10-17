//! Collection and organisation of peers.
//! 
//! The `PeerGroup` component is a marker identifying an entity as a 'group' of peers.
//! Peer groups use `bevy_hierarchy` for organisation. Therefore, the functions from the `BuildChildren` trait should be used to manage peer groups.
//! The `PeerGroups` systemparam can be used to view peer group related organisation.
//! 
//! An entity with `PeerGroup` can contain other groups.
//! When iterating the peers contained in a group, the peers contained in sub-groups are also returned.

use bevy::{prelude::*, ecs::system::SystemParam};
use crate::prelude::NetworkPeer;

/// Methods for viewing peer groups and network peers contained in them.
#[derive(SystemParam)]
pub struct PeerGroups<'w, 's> {
    all_groups: Query<'w, 's, Entity, With<PeerGroup>>,
    ancestor_groups: Query<'w, 's, &'static Children, With<PeerGroup>>,
    peers: Query<'w, 's, Entity, With<NetworkPeer>>,
}

impl PeerGroups<'_, '_> {
    /// Returns an iterator over all `NetworkPeer`s contained in `group`.
    pub fn peers_in_group(&self, group: Entity) -> () {
        todo!()
    }

    /// Returns an iterator over all `PeerGroup`s contained in `group`.
    pub fn groups_in_group(&self, group: Entity) -> () {
        todo!()
    }

    /// Returns `true` if this group contains `peer`.
    /// Also returns `false` if `group` doesn't exist or have a `Children` component.
    pub fn group_contains_peer(&self, group: Entity, peer: Entity) -> () {
        todo!()
    }
}

/// Marker component for an entity representing a 'group' of peers.
/// See [the module level documentation](self) for more.
#[derive(Component)]
pub struct PeerGroup;