use bevy::prelude::*;

/// The 'state' of a replicated object.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
#[reflect(Debug, Default, PartialEq, Hash)]
pub enum ReplicationPause {
    /// The component is being kept up to date on all peers.
    Active,

    /// The component is not being kept up to date, but is not being removed.
    Paused,

    /// Inherit replication state from a parent, if any.
    /// If there is no parent, acts as if set to [`Active`].
    #[default]
    Inherit,
}

/// How replication systems recurse through the entity hierarchy.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
#[reflect(Debug, Default, PartialEq, Hash)]
pub enum ReplicationRecursion {
    /// Recurse all children.
    #[default]
    Recurse,

    /// Don't recurse children.
    Single,

    /// Inherit behavior from a parent, if any.
    /// Defaults to `Recurse` if no parent exists.
    Inherit,
}