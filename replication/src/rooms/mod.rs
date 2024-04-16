mod systems;

use std::{collections::BTreeSet, marker::PhantomData, sync::Arc};
use bevy::{ecs::component::TableStorage, prelude::*};
use bevy_stardust::prelude::*;
use smallvec::{smallvec, SmallVec};
use crate::prelude::*;

/// Enables network room functionality.
/// Implicitly adds [`CoreReplicationPlugin`] if not present.
/// 
/// Must be added before typed plugins like:
/// - [`ReplicateResourcePlugin<T>`]
/// - [`ReplicateComponentPlugin<T>`]
pub struct ReplicationRoomsPlugin;

impl Plugin for ReplicationRoomsPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<CoreReplicationPlugin>() {
            app.add_plugins(CoreReplicationPlugin);
        }

        app.add_systems(PostUpdate, (
            systems::update_entity_cache,
        ).in_set(PostUpdateReplicationSystems::DetectChanges));
    }
}

/// Caches room memberships for components of type `T` for faster access.
/// This will only apply to rooms with the [`CacheMemberships<T>`](CacheMemberships) component.
/// 
/// Entity memberships themselves are always cached.
pub struct CacheRoomMembershipsPlugin<T: ReplicableComponent>(PhantomData<T>);

impl<T: ReplicableComponent> Plugin for CacheRoomMembershipsPlugin<T> {
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
/// If added to an entity, it affects all components.
/// 
/// If the component is not present for the relevant type,
/// filtering is not applied.
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
    pub filter: RoomFilterConfig,
    phantom: PhantomData<T>,
}

impl<T> NetworkRoomMembership<T> {
    /// Creates a new [`NetworkRoomFilter<T>`].
    pub fn new(filter: RoomFilterConfig) -> Self {
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

impl<T: ReplicableComponent> Component for NetworkRoomMembership<T> {
    type Storage = T::Storage;
}

impl Resource for NetworkRoomMembership<All> {}

impl<T: ReplicableResource> Resource for NetworkRoomMembership<T> {}

/// Special type argument for [`NetworkRoomFilter`].
/// See the documentation for more information.
#[derive(Debug, Clone, Copy, Reflect)]
#[reflect(Debug)]
pub struct All;

/// Filtering method.
pub enum RoomFilterConfig {
    /// Replicated to peers that are members of this group.
    InclusiveSingle(Entity),

    /// Replicated to peers that are members in at least one of the contained groups.
    InclusiveMany(SmallVec<[Entity; 4]>),

    /// Replicated to peers that are not members of this group.
    ExclusiveSingle(Entity),

    /// Replicated to peers that are not members of any of the contained groups.
    ExclusiveMany(SmallVec<[Entity; 4]>),

    /// Use a custom function for filtering.
    /// `true` means that the target is replicated in the room with the passed entity ID.
    CustomFunction(Arc<dyn Fn(Entity) -> bool + Send + Sync>)
}

impl RoomFilterConfig {
    fn rm_many(vec: &mut SmallVec<[Entity; 4]>, item: Entity) {
        let el = vec.iter()
            .enumerate()
            .filter(|(_, e)| **e == item)
            .map(|v| v.0)
            .collect::<SmallVec<[usize; 8]>>();

        for idx in el.iter() {
            vec.remove(*idx);
        }
    }

    /// Try to add `room` to the set such that in [`filter`](Self::filter) will return `true` when passed `room`.
    /// 
    /// Returns `false` if this was impossible.
    /// This currently only occurs if the variant is [`CustomFunction`][CustomFunction].
    /// 
    /// [CustomFunction]: RoomFilterConfig::CustomFunction
    pub fn include(&mut self, room: Entity) -> bool {
        match self {
            RoomFilterConfig::InclusiveSingle(prev) => {
                if *prev == room { return true; }
                *self = Self::InclusiveMany(smallvec![*prev, room]);
                return true;
            },
            RoomFilterConfig::InclusiveMany(vec) => {
                if vec.contains(&room) { return true; }
                vec.push(room);
                return true;
            },
            RoomFilterConfig::ExclusiveSingle(itm) => {
                if *itm != room { return true; }
                *self = Self::ExclusiveMany(smallvec![]);
                return true;
            },
            RoomFilterConfig::ExclusiveMany(vec) => {
                Self::rm_many(vec, room);
                return true;
            },
            RoomFilterConfig::CustomFunction(_) => {
                // Not possible
                return false;
            },
        }
    }

    /// Try to add `room` to the set such that in [`filter`](Self::filter) will return `false` when passed `room`.
    /// 
    /// Returns `false` if this was impossible.
    /// This currently only occurs if the variant is [`CustomFunction`][CustomFunction].
    /// 
    /// [CustomFunction]: RoomFilterConfig::CustomFunction
    pub fn exclude(&mut self, room: Entity) -> bool {
        match self {
            RoomFilterConfig::InclusiveSingle(prev) => {
                if *prev != room { return true; }
                *self = Self::InclusiveMany(smallvec![]);
                return true;
            },
            RoomFilterConfig::InclusiveMany(vec) => {
                Self::rm_many(vec, room);
                return true;
            },
            RoomFilterConfig::ExclusiveSingle(prev) => {
                if *prev == room { return true; }
                *self = Self::ExclusiveMany(smallvec![*prev, room]);
                return true;
            },
            RoomFilterConfig::ExclusiveMany(vec) => {
                if vec.contains(&room) { return true; }
                vec.push(room);
                return true;
            },
            RoomFilterConfig::CustomFunction(_) => {
                // Not possible
                return false;
            },
        }
    }

    /// Returns `true` if group matches the filter.
    pub fn filter(&self, group: Entity) -> bool {
        self.filter_inlined(group)
    }

    /// Returns `true` if `group` matches the filter.
    /// This function is inlined - use [`filter`](Self::filter) if you don't want this.
    #[inline]
    pub(crate) fn filter_inlined(&self, group: Entity) -> bool {
        // TODO: Maybe ensure vecs are sorted so binary search can be used
        match self {
            RoomFilterConfig::InclusiveSingle(val) => *val == group,
            RoomFilterConfig::InclusiveMany(set) => set.contains(&group),
            RoomFilterConfig::ExclusiveSingle(val) => *val != group,
            RoomFilterConfig::ExclusiveMany(set) => !set.contains(&group),
            RoomFilterConfig::CustomFunction(func) => func(group),
        }
    }
}

/// Cached component memberships for `T`.
/// This is added to entities with [`NetworkRoom`], not [`NetworkRoomMembership`].
/// 
/// Improves performance for entities that:
/// - Have `NetworkRoomMembership<T>`
/// - Are replicated to a large amount of peers
/// 
/// Does nothing if [CacheRoomMembershipsPlugin] isn't added.
#[derive(Default, Component)]
pub struct CacheMemberships<T: ReplicableComponent> {
    cache: BTreeSet<Entity>,
    phantom: PhantomData<T>,
}