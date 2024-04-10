use bevy::prelude::*;

/// The 'state' of a replicated object.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
#[reflect(Debug, Default, PartialEq, Hash)]
pub enum ReplicationState {
    /// The component is being kept up to date on all peers.
    Active,

    /// The component is not being kept up to date, but is not being removed.
    Paused,

    /// Inherit replication state from a parent, if any.
    /// If there is no parent, acts as if set to [`Active`].
    #[default]
    Inherit,
}