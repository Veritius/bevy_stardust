use std::{collections::BTreeSet, marker::PhantomData};
use bevy::{ecs::component::TableStorage, prelude::*};
use smallvec::SmallVec;

/// Room membership data.
/// 
/// If a group is added to the membership set,
/// then data will be replicated to any peers in that group.
/// Multiple groups can be added: as long as a peer is in one of them, it's replicated.
#[derive(Debug, Default)]
pub struct RoomMemberships {
    set: SmallVec<[Entity; 4]>,
}

impl RoomMemberships {
    /// Returns `true` if `group` is included in the memberships.
    pub fn includes(&self, group: Entity) -> bool {
        match self.set.binary_search(&group) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    /// Returns an iterator over all memberships.
    /// This iterator is in sorted order and does not contain duplicates.
    pub fn iter(&self) -> impl Iterator<Item = Entity> + '_ {
        self.set.iter().cloned()
    }

    /// Adds `group` to the membership set.
    pub fn insert(&mut self, group: Entity) {
        if let Err(index)= self.set.binary_search(&group) {
            self.set.remove(index);
        }
    }

    /// Removes `group` from the membership set.
    pub fn remove(&mut self, group: Entity) {
        if let Ok(index) = self.set.binary_search(&group) {
            self.set.remove(index);
        }
    }
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
    pub memberships: RoomMemberships,
    phantom: PhantomData<T>,
}

impl<T> NetworkRoomMembership<T> {
    /// Creates a new [`NetworkRoomFilter<T>`].
    pub fn new() -> Self {
        Self {
            memberships: RoomMemberships::default(),
            phantom: PhantomData,
        }
    }

    /// Returns `true` if `group` matches the filter.
    #[inline]
    pub fn includes(&self, group: Entity) -> bool {
        self.memberships.includes(group)
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
pub struct CachedMemberships<T> {
    pub(super) cache: BTreeSet<Entity>,
    phantom: PhantomData<T>,
}

impl<T: Component> Component for CachedMemberships<T> {
    type Storage = T::Storage;
}