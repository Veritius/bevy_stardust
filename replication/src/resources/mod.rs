//! Replication for resources.

mod messages;
mod systems;

use std::marker::PhantomData;
use bevy::prelude::*;
use bevy_stardust::prelude::*;
use crate::{prelude::*, serialisation::SerialisationFunctions};

/// Enables replicating the resource `T`.
/// 
/// This plugin must be added before [`StardustPlugin`].
/// Implicitly adds [`ReplicationPlugin`] if not present.
pub struct ResourceReplicationPlugin<T: Resource> {
    /// Functions used to serialise and deserialize `T`.
    /// See the [`SerialisationFunctions`] documentation for more information.
    pub serialisation: SerialisationFunctions<T>,

    /// The priority of network messages for replicating `T`.
    pub message_priority: u32,

    #[doc(hidden)]
    pub phantom: PhantomData<T>,
}

impl<T: Resource> Plugin for ResourceReplicationPlugin<T> {
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