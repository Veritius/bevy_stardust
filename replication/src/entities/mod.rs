mod components;
mod ids;
mod messages;
mod systems;

pub(crate) use ids::*;

pub use components::*;

use bevy::prelude::*;
use bevy_stardust::prelude::*;
use crate::prelude::*;

/// Adds entity replication functionality.
/// - Doesn't replicate hierarchies. Use [`HierarchyReplicationPlugin`].
/// - Doesn't replicate components. Use [`ComponentReplicationPlugin`].
pub struct EntityReplicationPlugin {
    /// The priority of network messages for entity replication.
    pub message_priority: u32,
}

impl Plugin for EntityReplicationPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ReplicateEntity>();
        
        app.add_channel::<messages::EntityReplicationChannel>(ChannelConfiguration {
            reliable: ReliabilityGuarantee::Reliable,
            ordered: OrderingGuarantee::Ordered,
            fragmented: false,
            priority: self.message_priority,
        });

        app.add_systems(PreUpdate, (
            systems::ensure_id_component,
            systems::receive_entity_messages,
        ).in_set(PreUpdateReplicationSystems::UpdateEntities).chain());
    }
}

/// Query filter for entities that are replicated over the network.
pub type Replicated = With<ReplicateEntity>;

/// Entities with this component will be replicated.
#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct ReplicateEntity {
    /// How each entity is referred to per peer.
    #[reflect(ignore)]
    pub(crate) ids: AssociatedNetworkIds,
}