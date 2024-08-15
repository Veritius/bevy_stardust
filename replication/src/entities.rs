//! Entity replication.

use std::collections::BTreeMap;
use aery::edges::RelationCommands;
use bevy::{ecs::system::EntityCommands, prelude::*};
use crate::{identifiers::{IdGenerator, NetId, Side}, modifiers::*};

/// Adds functionality for replicating entities.
pub struct EntityReplicationPlugin;

impl Plugin for EntityReplicationPlugin {
    fn build(&self, app: &mut App) {
        // Various observers for replication related events
        app.observe(replicated_component_removal_observer);
    }
}

/// Attached to entities to replicate them over the network.
#[derive(Debug, Component)]
pub struct Replicated;

/// A map of IDs attached to peers to store network identifiers.
#[derive(Component)]
pub(crate) struct ReplicatedEntityIds {
    generator: IdGenerator,
    fwd: BTreeMap<Entity, NetId>,
    bck: BTreeMap<NetId, Entity>,
}

impl ReplicatedEntityIds {
    pub fn new(side: Side) -> Self {
        Self {
            generator: IdGenerator::new(side),
            fwd: BTreeMap::new(),
            bck: BTreeMap::new(),
        }
    }

    pub fn insert(
        &mut self,
        entity: Entity,
        netid: NetId,
    ) {
        self.fwd.insert(entity, netid);
        self.bck.insert(netid, entity);
    }

    pub fn generate(
        &mut self,
        entity: Entity,
    ) -> NetId {
        let netid = self.generator.next_id();
        self.insert(entity, netid);
        return netid;
    }
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
    // Remove replication components
    entity.remove::<ReplicatedEntityIds>();

    // Remove replication relations
    entity.withdraw::<Visible>();
    entity.withdraw::<Hidden>();
    entity.withdraw::<Thawed>();
    entity.withdraw::<Frozen>();
}