//! Various types used to configure replication behaviors.

use bevy::prelude::*;

/// Inclusivity and exclusivity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum Clusivity {
    /// Exclude by default.
    Exclude,

    /// Include by default.
    Include,
}