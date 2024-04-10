use bevy::prelude::*;
use crate::*;

/// Query filter for entities that are replicated over the network.
pub type Replicated = With<ReplicateEntity>;

/// Entities with this component will be replicated.
#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct ReplicateEntity {
    /// See [`ReplicationState`]'s documentation.
    pub state: ReplicationState,
}

/// The descendants of this entity will be replicated, as long as the entity also has [`Replicated`].
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ReplicateDescendants;