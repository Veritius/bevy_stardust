use bevy::prelude::*;
use crate::prelude::*;

/// Query filter for entities that are replicated over the network.
pub type Replicated = With<ReplicateEntity>;

/// Entities with this component will be replicated.
#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct ReplicateEntity {
    /// Sets whether the replicated entity is 'paused'.
    pub pause: ReplicationPausingMode,
    pub(crate) computed_pause: bool,

    /// How entities in the hierarchy are replicated.
    /// 
    /// If a child in the hierarchy is found without [`ReplicateEntity`],
    /// the component is added with default values.
    pub hierarchy_mode: ReplicationHierarchyMode,
    pub(crate) computed_hierarchy_mode: bool,
}

/// How entities are 'paused' in replication, stopping updates but not despawning them.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
#[reflect(Debug, Default, PartialEq, Hash)]
pub enum ReplicationPausingMode {
    /// The component is being kept up to date on all peers.
    Enabled,

    /// The component is not being kept up to date, but is not being removed.
    Disabled,

    /// Inherit replication state from a parent, if any.
    /// If there is no parent, acts as if set to [`Active`].
    #[default]
    Inherit,
}

/// How child entities in a hierarchy are replicated.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
#[reflect(Debug, Default, PartialEq, Hash)]
pub enum ReplicationHierarchyMode {
    /// Replicates all children.
    Enabled,

    /// Doesn't replicated children.
    Disabled,

    /// Inherit the replication mode from a parent, if any.
    /// Defaults to [`Enabled`](ReplicationHierarchyMode::Enabled) if no parent exists.
    #[default]
    Inherit,
}