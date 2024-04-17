//! Organisation of network peers.

use bevy::prelude::*;
use smallvec::SmallVec;

/// A collection of network peers, used for organisational purposes.
/// 
/// This can be used for anything, such as teams of players, rooms for replication, or administrative permissions.
#[derive(Debug, Component, Reflect)]
#[reflect(Debug, Component)]
pub struct NetworkGroup(pub(crate) SmallVec<[Entity; 8]>);

impl Default for NetworkGroup {
    fn default() -> Self {
        Self(SmallVec::default())
    }
}

impl NetworkGroup {
    /// Adds the peer to the network group.
    /// Does nothing if the peer is already included.
    pub fn add(&mut self, peer: Entity) {
        match self.0.binary_search(&peer) {
            Ok(_) => {},
            Err(idx) => self.0.insert(idx, peer),
        }
    }

    /// Removes the peer from the network group.
    /// Does nothing if the peer isn't present.
    pub fn remove(&mut self, peer: Entity) {
        match self.0.binary_search(&peer) {
            Ok(idx) => { self.0.remove(idx); },
            Err(_) => {},
        }
    }

    /// Returns `true` if the peer is part of the network group.
    pub fn contains(&self, peer: Entity) -> bool {
        match self.0.binary_search(&peer) {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}