use bevy::prelude::*;

/// How child entities in a hierarchy are replicated.
#[derive(Debug, Default, Component, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
#[reflect(Debug, Default, Component, PartialEq, Hash)]
pub enum ReplicateHierarchy {
    /// Automatically replicates all children.
    Enabled,

    /// Doesn't replicate children.
    Disabled,

    /// Inherit the replication mode from a parent, if any.
    /// Defaults to [`Enabled`](ReplicateHierarchy::Enabled) if no parent exists.
    #[default]
    Inherit,
}