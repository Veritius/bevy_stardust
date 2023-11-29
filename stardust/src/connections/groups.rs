//! Organisation of network peers.

use bevy::prelude::*;
use smallvec::SmallVec;

/// A collection of network peers.
/// If used as a `target` in a `NetworkWriter` this will send to all the ids in the vector.
#[derive(Debug, Component)]
pub struct NetworkGroup(pub SmallVec<[Entity; 8]>);