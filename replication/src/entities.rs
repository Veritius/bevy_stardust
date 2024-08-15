//! Entity replication.

use aery::edges::RelationCommands;
use bevy::{ecs::system::EntityCommands, prelude::*};
use crate::{config::Clusivity, identifiers::NetId, modifiers::*};

/// Adds functionality for replicating entities.
pub struct EntityReplicationPlugin {
    /// Whether or not to replicate entities by default.
    pub opt: Clusivity,
}

impl Plugin for EntityReplicationPlugin {
    fn build(&self, app: &mut App) {
        // Various observers for replication related events
        app.observe(replicated_component_removal_observer);
    }
}

/// Attached to entities to replicate them over the network.
#[derive(Debug, Component)]
pub struct Replicated {
    id: NetId,
}

fn replicated_component_removal_observer(
    trigger: Trigger<OnRemove, Replicated>,
    mut commands: Commands,
) {
    // Get commands for the target entity
    let commands = commands.entity(trigger.entity());
    clear_replication_components(commands);
}

fn clear_replication_components(
    mut entity: EntityCommands,
) {
    // Remove replication relations
    entity.withdraw::<Visible>();
    entity.withdraw::<Hidden>();
    entity.withdraw::<Thawed>();
    entity.withdraw::<Frozen>();
}