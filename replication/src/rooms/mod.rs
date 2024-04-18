//! Replication room functionality.

mod membership;
mod params;
mod systems;

pub use membership::*;
pub use params::*;

use std::{collections::BTreeSet, marker::PhantomData};
use bevy::prelude::*;
use bevy_stardust::prelude::*;
use crate::prelude::*;

#[derive(Resource)]
struct RoomsEnabled;

/// Enables scoped replication using network rooms.
pub struct ScopedReplicationPlugin;

impl Plugin for ScopedReplicationPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<CoreReplicationPlugin>() {
            app.add_plugins(CoreReplicationPlugin);
        }

        app.register_type::<NetworkRoom>();

        app.insert_resource(RoomsEnabled);

        app.add_systems(PostUpdate, (
            systems::update_entity_cache,
        ).in_set(PostUpdateReplicationSystems::DetectChanges));
    }
}

/// Caches room memberships for components of type `T` for faster access.
/// This will only apply to rooms with the [`CacheMemberships<T>`](CacheMemberships) component.
/// 
/// Entity memberships themselves are always cached.
pub struct CacheRoomMembershipsPlugin<T: Component>(PhantomData<T>);

impl<T: Component> Plugin for CacheRoomMembershipsPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, systems::update_component_cache::<T>
            .in_set(PostUpdateReplicationSystems::DetectChanges));
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
    pub(crate) cache: BTreeSet<Entity>,
}

/// A bundle for a minimal network room.
#[derive(Bundle)]
#[allow(missing_docs)]
pub struct NetworkRoomBundle {
    pub room: NetworkRoom,
    pub group: NetworkGroup,
}