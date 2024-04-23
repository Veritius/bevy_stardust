//! Replication for resources.

mod change;
mod messages;
mod systems;

pub use change::NetRes;

use bevy::prelude::*;
use bevy_stardust::prelude::*;
use crate::{prelude::*, serialisation::SerialisationFunctions};

/// Supertrait for traits needed to replicate resources.
pub trait ReplicableResource: TypePath + Resource {}
impl<T: TypePath + Resource> ReplicableResource for T {}

/// Enables replicating the resource `T`.
/// 
/// This plugin must be added before [`StardustPlugin`].
pub struct ResourceReplicationPlugin<T: ReplicableResource> {
    /// Functions used to serialise and deserialize `T`.
    /// See the [`SerialisationFunctions`] documentation for more information.
    pub serialisation: SerialisationFunctions<T>,

    /// The priority of network messages for replicating `T`.
    pub message_priority: u32,
}

impl<T: ReplicableResource> Plugin for ResourceReplicationPlugin<T> {
    fn build(&self, app: &mut App) {
        app.insert_resource(messages::ResourceSerialisationFunctions {
            fns: self.serialisation.clone()
        });

        app.add_channel::<messages::ResourceReplicationMessages<T>>(ChannelConfiguration {
            reliable: ReliabilityGuarantee::Reliable,
            ordered: OrderingGuarantee::Sequenced,
            fragmented: true,
            priority: self.message_priority,
        });

        app.add_systems(PreUpdate, systems::recv_resource_data_system::<T>
            .in_set(ReplicationSystems::UpdateResources));

        app.add_systems(PostUpdate, systems::send_resource_data_system::<T>
            .in_set(ReplicationSystems::UpdateResources));
    }
}