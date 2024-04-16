use bevy::prelude::*;
use crate::prelude::*;

/// Query filter for entities that are replicated over the network.
pub type Replicated = With<ReplicateEntity>;

/// Entities with this component will be replicated.
#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct ReplicateEntity;

/// How child entities in a hierarchy are replicated.
#[derive(Debug, Default, Component, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
#[reflect(Debug, Default, Component, PartialEq, Hash)]
pub enum ReplicateHierarchy {
    /// Automatically replicates all children.
    Enabled,

    /// Doesn't replicated children.
    Disabled,

    /// Inherit the replication mode from a parent, if any.
    /// Defaults to [`Enabled`](ReplicateHierarchy::Enabled) if no parent exists.
    #[default]
    Inherit,
}