//! Replication for components.

mod messages;
mod systems;

use std::marker::PhantomData;
use bevy::prelude::*;
use bevy_stardust::prelude::*;
use crate::{prelude::*, serialisation::SerialisationFunctions};

/// Supertrait for traits needed to replicate components.
pub trait ReplicableComponent: TypePath + Component {}
impl<T: TypePath + Component> ReplicableComponent for T {}

/// Enables replicating the component `T` on entities.
pub struct ComponentReplicationPlugin<T: ReplicableComponent> {
    /// Functions used to serialise and deserialize `T`.
    /// See the [`SerialisationFunctions`] documentation for more information.
    pub serialisation: SerialisationFunctions<T>,

    /// The priority of network messages for replicating `T`.
    pub message_priority: u32,

    #[doc(hidden)]
    pub phantom: PhantomData<T>,
}

impl<T: ReplicableComponent> Plugin for ComponentReplicationPlugin<T> {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<EntityReplicationPlugin>() {
            panic!("ComponentReplicationPlugin must be added after EntityReplicationPlugin");
        }

        app.insert_resource(messages::ComponentSerialisationFunctions {
            fns: self.serialisation.clone()
        });

        app.add_channel::<messages::ComponentReplicationMessages<T>>(ChannelConfiguration {
            reliable: ReliabilityGuarantee::Reliable,
            ordered: OrderingGuarantee::Sequenced,
            fragmented: true,
            priority: self.message_priority,
        });
    }
}