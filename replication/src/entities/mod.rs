mod hierarchy;
mod ids;

pub(crate) use ids::*;

pub use hierarchy::*;

use bevy::prelude::*;

/// Query filter for entities that are replicated over the network.
pub type Replicated = With<ReplicateEntity>;

/// Entities with this component will be replicated.
#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct ReplicateEntity;