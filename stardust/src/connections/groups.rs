//! Organisation of network peers.

use bevy_ecs::prelude::*;
use smallvec::SmallVec;

/// A collection of network peers. This can be used for anything, such as teams of players, rooms for replication, or administrative permissions.
///
/// If used as a `target` in a `NetworkWriter` this will send to all the ids contained inside it.
#[derive(Debug, Component)]
pub struct NetworkGroup(pub SmallVec<[Entity; 8]>);