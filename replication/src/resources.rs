//! Resource replication.

use bevy::{ecs::schedule::InternedScheduleLabel, prelude::*};
use crate::serialisation::SerialisationFns;

/// Adds functionality for replicating resources.
pub struct ResourceReplicationPlugin<T> {
    /// The schedule in which changes from remote peers are applied.
    /// Defaults to [`PreUpdate`] if set to `None`.
    pub recv_schedule: Option<InternedScheduleLabel>,

    /// The schedule in which remote peers are informed of changes.
    /// Defaults to [`PostUpdate`] if set to `None`.
    pub send_schedule: Option<InternedScheduleLabel>,

    /// Functions used to serialise and deserialise `T`.
    pub serialise_fns: SerialisationFns<T>,
}

impl<T> Plugin for ResourceReplicationPlugin<T>
where
    T: Resource,
{
    fn build(&self, app: &mut App) {

    }
}