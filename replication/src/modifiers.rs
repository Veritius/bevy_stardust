//! 'Modifiers' that can be attached to replicated components and resources to change their behavior.

use std::{collections::BTreeSet, marker::PhantomData};
use bevy::{ecs::component::StorageType, prelude::*};
use crate::config::Clusivity;

// /// Overrides the state of `T` that is replicated to other peers,
// /// while keeping a hidden local state. Only effective if
// /// this application is the authority over the target.
// /// 
// /// This is a tool to *intentionally* report incorrect game state to peers.
// /// `Override` is very niche and should be used carefully, as it can cause
// /// various hard-to-debug problems, such as with prediction.
// /// 
// /// Automatically removed if `T` is removed.
// pub struct Override<T> {
//     /// The inner value.
//     pub inner: T,
// }

// impl<T: Component> Component for Override<T> {
//     const STORAGE_TYPE: StorageType = T::STORAGE_TYPE;
// }

// impl<T: Resource> Resource for Override<T> {}

/// Prevents changes to `T` from being replicated to certain peers.
/// 
/// Automatically removed if `T` is removed.
#[derive(Debug)]
pub struct Freeze<T> {
    cls: Clusivity,
    set: BTreeSet<Entity>,
    _p1: PhantomData<T>,
}

impl<T> Freeze<T> {
    /// Creates a new `Freeze` component with a given [`Clusivity`].
    pub fn new(clusivity: Clusivity) -> Self {
        Self {
            cls: clusivity,
            set: BTreeSet::default(),
            _p1: PhantomData,
        }
    }

    /// Prevent `peer` from receiving updates.
    pub fn refuse(&mut self, peer: Entity) {
        match self.cls {
            Clusivity::Exclude => self.set.insert(peer),
            Clusivity::Include => self.set.remove(&peer),
        };
    }

    /// Allow `peer` to receive updates.
    pub fn allow(&mut self, peer: Entity) {
        match self.cls {
            Clusivity::Exclude => self.set.remove(&peer),
            Clusivity::Include => self.set.insert(peer),
        };
    }

    /// Returns `true` if `peer` receives updates.
    pub fn allowed(&self, peer: Entity) -> bool {
        match self.cls {
            Clusivity::Exclude => !self.set.contains(&peer),
            Clusivity::Include => self.set.contains(&peer),
        }
    }
}

impl<T: Component> Component for Freeze<T> {
    const STORAGE_TYPE: StorageType = T::STORAGE_TYPE;
}

impl<T: Resource> Resource for Freeze<T> {}