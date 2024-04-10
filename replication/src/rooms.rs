use std::{marker::PhantomData, sync::Arc};
use bevy::{ecs::component::TableStorage, prelude::*};
use bevy_stardust::prelude::*;
use smallvec::SmallVec;
use crate::*;

/// Defines a 'network room' entity. This filters the entities that are replicated to each peer.
///
/// Peers considered members of the room (as per [`NetworkGroup`]) will have entities replicated to them.
#[derive(Debug, Component, Reflect)]
#[reflect(Debug, Component)]
pub struct NetworkRoom;

/// A bundle for a minimal network room.
#[derive(Bundle)]
#[allow(missing_docs)]
pub struct NetworkRoomBundle {
    pub room: NetworkRoom,
    pub group: NetworkGroup,
}

/// Controls how rooms affect the replication of type `T`.
/// 
/// This type is both a [`Resource`] and [`Component`].
/// When added to the [`World`] or an [`Entity`], it affects how they are replicated.
/// 
/// By default, `T` is [`All`], making it affect all replicated values.
/// If added to the World, it affects all resources.
/// If added to an entity, it affects all components.
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
pub struct NetworkRoomFilter<T = All> {
    /// The inner filter method.
    pub filter: RoomFilterConfig,
    phantom: PhantomData<T>,
}

impl<T> NetworkRoomFilter<T> {
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

impl Component for NetworkRoomFilter<All> {
    type Storage = TableStorage;
}

impl<T: ReplicableComponent> Component for NetworkRoomFilter<T> {
    type Storage = T::Storage;
}

impl Resource for NetworkRoomFilter<All> {}

impl<T: ReplicableResource> Resource for NetworkRoomFilter<T> {}

/// Special type argument for [`NetworkRoomFilter`].
/// See the documentation for more information.
#[derive(Debug, Clone, Copy, Reflect)]
#[reflect(Debug)]
pub struct All;

/// Filtering method.
pub enum RoomFilterConfig {
    /// Replicated to peers that are members of this group.
    InclusiveSingle(Entity),
    /// Replicated to peers that are members in at least 1 group.
    InclusiveMany(SmallVec<[Entity; 4]>),
    /// Replicated to peers that are not members of this group.
    ExclusiveSingle(Entity),
    /// Replicated to peers that are not members of any of the contained groups.
    ExclusiveMany(SmallVec<[Entity; 4]>),
    /// Use a custom function for filtering.
    /// `true` means that the target is replicated.
    CustomFunc(Arc<dyn Fn(Entity) -> bool + Send + Sync>)
}

impl RoomFilterConfig {
    /// Returns `true` if `group` matches the filter.
    pub fn filter(&self, group: Entity) -> bool {
        match self {
            RoomFilterConfig::InclusiveSingle(val) => *val == group,
            RoomFilterConfig::InclusiveMany(set) => set.contains(&group),
            RoomFilterConfig::ExclusiveSingle(val) => *val != group,
            RoomFilterConfig::ExclusiveMany(set) => !set.contains(&group),
            RoomFilterConfig::CustomFunc(func) => func(group),
        }
    }
}