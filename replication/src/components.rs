use std::marker::PhantomData;
use bevy::prelude::*;
use bevy_stardust::prelude::*;
use crate::plugin::*;
use crate::traits::*;

/// Trait for components that can be replicated.
/// Automatically implemented for types that satisfy the requirements.
pub trait ReplicableComponent: Component + Replicable {}
impl<T> ReplicableComponent for T where T: Component + Replicable {}

/// Enables replicating the component `T`.
pub struct ReplicateComponentPlugin<T: ReplicableComponent>(PhantomData<T>);

impl<T: ReplicableComponent> Plugin for ReplicateComponentPlugin<T> {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<ReplicationPlugin>() {
            panic!("ReplicationPlugin must be added before ReplicateComponentPlugin")
        }
    }
}

impl<T: ReplicableComponent> Default for ReplicateComponentPlugin<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

/// Entities with this component will be replicated.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Replicated;

/// The descendants of this entity will be replicated, as long as the entity also has [`Replicated`].
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ReplicateDescendants;