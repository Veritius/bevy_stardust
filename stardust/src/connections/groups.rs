//! Collection and organisation of peers.

use bevy::prelude::*;

/// Marker component for an entity representing a 'group' of peers.
/// See [the module level documentation](self) for more.
#[derive(Component)]
pub struct PeerGroup {
    members: Vec<Entity>,
}