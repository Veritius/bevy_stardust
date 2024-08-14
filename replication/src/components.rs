//! Component replication.

use std::any::type_name;
use bevy::{ecs::schedule::InternedScheduleLabel, prelude::*};
use crate::{changes::NetChangeState, entities::EntityReplicationPlugin, serialisation::SerialisationFns};

/// Adds functionality for replicating components.
/// 
/// Requires [`EntityReplicationPlugin`] to be added beforehand.
pub struct ComponentReplicationPlugin<T> {
    /// The schedule in which changes from remote peers are applied.
    /// Defaults to [`PreUpdate`] if set to `None`.
    pub recv_schedule: Option<InternedScheduleLabel>,

    /// The schedule in which remote peers are informed of changes.
    /// Defaults to [`PostUpdate`] if set to `None`.
    pub send_schedule: Option<InternedScheduleLabel>,

    /// Functions used to serialise and deserialise `T`.
    pub serialise_fns: SerialisationFns<T>,
}

impl<T> Plugin for ComponentReplicationPlugin<T>
where
    T: Component,
{
    fn build(&self, app: &mut App) {
        assert!(app.is_plugin_added::<crate::entities::EntityReplicationPlugin>(),
            "{} requires {}, but it was not added", type_name::<Self>(), type_name::<EntityReplicationPlugin>());

        // Various observers for replication related events
        app.observe(replicated_component_removal_observer::<T>);
    }
}

fn replicated_component_removal_observer<T: Component>(
    trigger: Trigger<OnRemove, T>,
    mut commands: Commands,
) {
    // We do not care about placeholders
    if trigger.entity() == Entity::PLACEHOLDER { return }

    // Remove any replication-related components
    commands.entity(trigger.entity())
        .remove::<NetChangeState<T>>();
}