//! Replication rooms.

use bevy::prelude::*;
use aery::prelude::*;
use crate::config::Clusivity;

/// Adds support for [replication rooms](ReplicationRoom).
pub struct RoomsPlugin;

impl Plugin for RoomsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ReplicationRoom>();
    }
}

/// A [`Relation`] identifying a [peer](bevy_stardust::connections) as a member of a [`ReplicationRoom`].
#[derive(Relation)]
pub struct RoomMember;

/// A replication room, allowing configuration to be applied to many peers at once.
#[derive(Debug, Component, Reflect)]
#[reflect(Component)]
pub struct ReplicationRoom {
    /// Whether related members are considered excluded or included by their relation.
    pub clusivity: Clusivity,
}