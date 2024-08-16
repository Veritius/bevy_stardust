//! Controls for what entities are visible to which peers.

use std::{collections::BTreeSet, marker::PhantomData};
use bevy::{ecs::component::StorageType, prelude::*};
use aery::prelude::*;

/// Adds functionality for defining scopes, which control which peers can see what entities.
pub struct EntityScopePlugin;

impl Plugin for EntityScopePlugin {
    fn build(&self, app: &mut App) {

    }
}

/// An [entity relation](aery) that allows an entity to be replicated to a peer.
/// Targeted from the entity to be replicated (the host) to the peer (the target).
#[derive(Relation)]
pub struct Visible<T = Entity>(PhantomData<T>);

/// An [entity relation](aery) that prevents an entity from being replicated to a peer.
/// Targeted from the entity to be replicated (the host) to the peer (the target).
#[derive(Relation)]
pub struct Hidden<T = Entity>(PhantomData<T>);

/// An [entity relation](aery) that makes the entity inherit the visibility of its target.
/// Targeted from the inheritor (the host) to the inherited (the target).
pub struct Inherit<T = Entity>(PhantomData<T>);

/// Added to entities to cache which peers can see this entity.
#[derive(Debug)]
pub struct VisibilityCache<T = Entity> {
    cache: BTreeSet<Entity>,
    _p: PhantomData<T>,
}

impl<T> VisibilityCache<T> {
    /// Creates a new empty [`VisibilityCache`].
    /// Automatically populated when inserted into the world.
    pub fn new() -> Self {
        Self {
            cache: BTreeSet::new(),
            _p: PhantomData,
        }
    }
}

impl<T: Send + Sync + 'static> Component for VisibilityCache<T> {
    const STORAGE_TYPE: StorageType = StorageType::Table;
}