use bevy::prelude::*;
use crate::*;

/// Query filter for entities that are replicated over the network.
pub type Replicated = With<ReplicateEntity>;

/// Entities with this component will be replicated.
#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct ReplicateEntity {
    /// See [`ReplicationPause`]'s documentation.
    pub paused: ReplicationPause,
    pub(crate) computed: bool,
}

/// The descendants of this entity will be replicated, as long as the entity with this component also has [`Replicated`].
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ReplicateDescendants;