use bevy::prelude::*;

/// Entities with this component will be replicated.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ReplicateEntity;

/// The descendants of this entity will be replicated, as long as the entity also has [`Replicated`].
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ReplicateDescendants;