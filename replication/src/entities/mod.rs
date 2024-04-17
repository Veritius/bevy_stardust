mod components;
mod hierarchy;
mod ids;

pub(crate) use ids::*;

pub use components::*;
pub use hierarchy::*;

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
        
        app.add_channel::<EntityReplicationChannel>(ChannelConfiguration {
            reliable: ReliabilityGuarantee::Reliable,
            ordered: OrderingGuarantee::Ordered,
            fragmented: false,
            priority: self.message_priority,
        });
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

/// Stardust channel for entity replication.
#[derive(Default)]
pub(crate) struct EntityReplicationChannel;