use std::marker::PhantomData;
use bevy::{ecs::component::TableStorage, prelude::*};
use bevy_stardust::prelude::*;
use crate::*;

/// Defines a 'network room' entity. This filters the entities that are replicated to each peer.
///
/// Peers considered members of the room (as per [`NetworkGroup`]) will have entities replicated to them.
#[derive(Debug, Component, Reflect)]
#[reflect(Debug, Component)]
pub struct NetworkRoom {
    /// See [`RoomFilterMode`]'s documentation.
    pub filter: RoomFilterMode,
}

/// A bundle for a minimal network room.
#[derive(Bundle)]
#[allow(missing_docs)]
pub struct NetworkRoomBundle {
    pub room: NetworkRoom,
    pub group: NetworkGroup,
}

/// Defines how peers in the room should be filtered out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
#[reflect(Debug, PartialEq)]
pub enum RoomFilterMode {
    /// Only peers in the room will have entities replicated to them.
    Exclusive,

    /// Only peers outside of the room will have entities replicated to them.
    Inclusive,
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
pub struct NetworkRoomFilter<T = All>  {
    phantom: PhantomData<T>,
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