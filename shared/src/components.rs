use std::marker::PhantomData;

use bevy::prelude::Component;

/// Added to entities to mark them as replicated.
#[derive(Component)]
pub struct ReplicatedEntity;

/// Enables replication for disabled-by-default components.
#[derive(Component)]
pub struct AllowReplication<T: Component>(PhantomData<T>);
impl<T: Component> AllowReplication<T> { fn new() -> Self { Self(PhantomData) }}

/// Disables replication for enabled-by-default components.
#[derive(Component)]
pub struct PreventReplication<T: Component>(PhantomData<T>);
impl<T: Component> PreventReplication<T> { fn new() -> Self { Self(PhantomData) }}