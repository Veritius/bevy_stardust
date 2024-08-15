//! Replication groups.

use bevy::prelude::*;
use aery::prelude::*;
use crate::config::Clusivity;

/// Adds support for [replication groups](ReplicationGroup).
pub struct ReplicationGroupsPlugin;

impl Plugin for ReplicationGroupsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ReplicationGroup>();
    }
}

/// A [`Relation`] identifying a [peer](bevy_stardust::connections) as a member of a [`ReplicationGroup`].
#[derive(Relation)]
pub struct GroupMember;

/// A replication group, allowing configuration to be applied to many peers at once.
#[derive(Debug, Component, Reflect)]
#[reflect(Component)]
pub struct ReplicationGroup {
    /// Whether related members are considered excluded or included by their relation.
    pub clusivity: Clusivity,
}