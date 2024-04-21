//! Replication room functionality.

mod caching;
mod membership;
mod params;

pub use membership::*;
pub use params::*;

use std::{collections::BTreeSet, marker::PhantomData};
use bevy::prelude::*;
use bevy_stardust::prelude::*;
use crate::prelude::*;

/// Allows enabling/disabling replication scoping while the app is running.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, States, Reflect)]
#[reflect(Debug, Default, PartialEq, Hash)]
pub enum UseReplicationScope {
    /// Enables replication scoping.
    #[default]
    Enabled,

    /// Disables replication scoping.
    /// Caches will continue to be updated.
    Frozen,

    /// Disables replication scoping.
    /// Caches will not be updated.
    /// 
    /// Changing from this state will incur a significant performance cost as caches are updated.
    Disabled,
}

impl UseReplicationScope {
    /// Returns `true` if replication scope is [`Enabled`](UseReplicationScope::Enabled).
    #[inline]
    pub fn is_enabled(&self) -> bool {
        *self == Self::Enabled
    }
}

/// Enables scoped replication using network rooms.
/// 
/// Whether scope is enabled or disabled can be controlled with [`ReplicationScoping`].
pub struct ScopedReplicationPlugin;

impl Plugin for ScopedReplicationPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<CoreReplicationPlugin>() {
            app.add_plugins(CoreReplicationPlugin);
        }

        app.register_type::<NetworkRoom>();
        app.register_type::<UseReplicationScope>();

        app.init_state::<UseReplicationScope>();

        app.add_systems(PostUpdate, (
            caching::cache_update_system::<NetworkRoom, All>,
        ).in_set(PostUpdateReplicationSystems::DetectChanges));
    }
}

/// Defines a 'network room' entity. This filters the entities that are replicated to each peer.
///
/// Peers considered members of the room (as per [`NetworkGroup`]) will have entities replicated to them.
#[derive(Debug, Default, Component, Reflect)]
#[reflect(Debug, Default, Component)]
pub struct NetworkRoom {
    // Cached memberships for entities.
    #[reflect(ignore)]
    cache: BTreeSet<Entity>,
}

/// A bundle for a minimal network room.
#[derive(Bundle)]
#[allow(missing_docs)]
pub struct NetworkRoomBundle {
    pub room: NetworkRoom,
    pub group: NetworkGroup,
}