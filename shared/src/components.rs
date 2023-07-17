use bevy::prelude::Component;
use crate::types::NetworkEntityId;

/// Added to entities to mark them as replicated.
#[derive(Component)]
pub struct ReplicatedEntity {
    network_id: NetworkEntityId,
}