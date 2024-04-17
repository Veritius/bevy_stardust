mod filter;
mod systems;

pub use filter::MembershipFilter;

use std::{collections::BTreeSet, marker::PhantomData};
use bevy::{ecs::component::TableStorage, prelude::*};
use bevy_stardust::prelude::*;
use crate::prelude::*;

/// Enables scoped replication using network rooms.
pub struct ScopedReplicationPlugin;

impl Plugin for ScopedReplicationPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<CoreReplicationPlugin>() {
            app.add_plugins(CoreReplicationPlugin);
        }

        app.register_type::<NetworkRoom>();

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

/// Controls how rooms affect replication.
/// 
/// This type is both a [`Resource`] and [`Component`].
/// When added to the [`World`] or an [`Entity`], it affects how they are replicated.
/// 
/// By default, `T` is [`All`], making it affect all replicated values.
/// If added to the World, it affects all resources.
/// If added to an entity, it affects the entity itself.
/// 
/// If the component is not present for the relevant type,
/// filtering is not applied, and the component will be replicated to all peers.
/// 
/// ## Precedence
/// `T` takes precedence over [`All`] and will override it.
/// For `T`, the value of `Self<T>` will be used instead of `Self<All>`.
/// 
/// | `Self<All>` | `Self<T>` | Precedence  |
/// | ----------- | --------- | ----------- |
/// | Yes         | No        | `Self<All>` |
/// | Yes         | Yes       | `Self<T>`   |
/// | No          | Yes       | `Self<T>`   |
/// | No          | No        | Neither     |
pub struct NetworkRoomMembership<T: ?Sized = All> {
    /// The inner filter method.
    pub filter: MembershipFilter,
    phantom: PhantomData<T>,
}

impl<T> NetworkRoomMembership<T> {
    /// Creates a new [`NetworkRoomFilter<T>`].
    pub fn new(filter: MembershipFilter) -> Self {
        Self {
            filter,
            phantom: PhantomData,
        }
    }

    /// Returns `true` if `group` matches the filter.
    /// Inlines to [`RoomFilterConfig::filter`].
    #[inline]
    pub fn filter(&self, group: Entity) -> bool {
        self.filter.filter(group)
    }
}

impl Component for NetworkRoomMembership<All> {
    type Storage = TableStorage;
}

impl<T: Component> Component for NetworkRoomMembership<T> {
    type Storage = T::Storage;
}

impl Resource for NetworkRoomMembership<All> {}

impl<T: Resource> Resource for NetworkRoomMembership<T> {}

/// Special type argument for [`NetworkRoomFilter`].
/// See the documentation for more information.
#[derive(Debug, Clone, Copy, Reflect)]
#[reflect(Debug)]
pub struct All;

/// Caches room memberships for the component `T`.
/// Improves iteration performance for entities with [`NetworkRoomMembership<T>`].
/// This comes at an additional cost of mutating or adding the [`NetworkRoomMembership<T>`] component.
/// 
/// This should be added to entities with the [`NetworkRoom`] component,
/// and not the [`NetworkRoomMembership`] component. Does nothing if
/// [`CacheRoomMembershipsPlugin`] isn't added.
#[derive(Default)]
pub struct CacheMemberships<T> {
    cache: BTreeSet<Entity>,
    phantom: PhantomData<T>,
}

impl<T: Component> Component for CacheMemberships<T> {
    type Storage = T::Storage;
}