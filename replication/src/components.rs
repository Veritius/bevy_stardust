//! Component replication.

use std::any::type_name;
use aery::edges::RelationCommands;
use bevy::{ecs::{schedule::InternedScheduleLabel, system::EntityCommands}, prelude::*};
use crate::{changes::NetChangeState, config::Clusivity, entities::{EntityReplicationPlugin, Replicated}, modifiers::*, serialisation::SerialisationFns};

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

    /// When to replicate.
    pub opt: Clusivity,
}

impl<T> Plugin for ComponentReplicationPlugin<T>
where
    T: Component,
{
    fn build(&self, app: &mut App) {
        assert!(app.is_plugin_added::<crate::entities::EntityReplicationPlugin>(),
            "{} requires {}, but it was not added", type_name::<Self>(), type_name::<EntityReplicationPlugin>());

        // Various observers for replication related events
        app.observe(entity_replication_removal_observer::<T>);
        app.observe(component_replication_removal_observer::<T>);
    }
}

fn entity_replication_removal_observer<T: Component>(
    trigger: Trigger<OnRemove, Replicated>,
    mut commands: Commands,
) {
    // Get commands for the target entity
    let commands = commands.entity(trigger.entity());
    clear_replication_components::<T>(commands);
}

fn component_replication_removal_observer<T: Component>(
    trigger: Trigger<OnRemove, T>,
    query: Query<&Replicated, With<T>>,
    mut commands: Commands,
) {
    // If it's not in the query, we don't care
    if !query.contains(trigger.entity()) { return }

    // Get commands for the target entity
    let commands = commands.entity(trigger.entity());
    clear_replication_components::<T>(commands);
}

fn clear_replication_components<T: Component>(
    mut entity: EntityCommands,
) {
    // Remove replication components
    entity.remove::<NetChangeState<T>>();

    // Remove replication relations
    entity.withdraw::<Visible<T>>();
    entity.withdraw::<Hidden<T>>();
    entity.withdraw::<Thawed<T>>();
    entity.withdraw::<Frozen<T>>();
}