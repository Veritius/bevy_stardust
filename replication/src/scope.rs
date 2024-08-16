//! Controls for what entities are visible to which peers.

use std::{collections::BTreeSet, marker::PhantomData};
use bevy::{ecs::component::StorageType, prelude::*};
use aery::prelude::*;
use crate::entities::Replicated;

/// An [entity relation](aery) that allows an entity to be replicated to a peer.
/// Targeted from the peer (the host) to the entity being replicated (the target).
#[derive(Relation)]
pub struct Visible<T = Entity>(PhantomData<T>);

/// An [entity relation](aery) that prevents an entity from being replicated to a peer.
/// Targeted from the peer (the host) to the entity being replicated (the target).
#[derive(Relation)]
pub struct Hidden<T = Entity>(PhantomData<T>);

/// An [entity relation](aery) that makes the entity visible if any of its targets are visible.
#[derive(Relation)]
pub struct Connect<T = Entity>(PhantomData<T>);

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

/// Adds functionality for defining scopes, which control which peers can see what entities.
pub struct EntityScopePlugin;

impl Plugin for EntityScopePlugin {
    fn build(&self, app: &mut App) {
        // Various observers
        app.observe(visible_relation_removed_observer);
        app.observe(hidden_relation_removed_observer);
        app.observe(connect_relation_removed_observer);
    }
}

fn visible_relation_removed_observer(
    trigger: Trigger<UnsetEvent<Visible>>,
    mut caches: Query<&mut VisibilityCache, With<Replicated>>,
) {

}

fn hidden_relation_removed_observer(
    trigger: Trigger<UnsetEvent<Hidden>>,
    mut caches: Query<&mut VisibilityCache, With<Replicated>>,
) {

}

fn connect_relation_removed_observer(
    trigger: Trigger<UnsetEvent<Connect>>,
    mut caches: Query<&mut VisibilityCache, With<Replicated>>,
) {

}