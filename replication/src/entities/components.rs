use std::marker::PhantomData;
use bevy::prelude::*;
use bevy_stardust::prelude::*;
use crate::{prelude::*, serialisation::SerialisationFunctions};

/// Enables replicating the component `T` on entities.
pub struct ComponentReplicationPlugin<T: Component> {
    /// Functions used to serialise and deserialize `T`.
    /// See the [`SerialisationFunctions`] documentation for more information.
    pub serialisation: SerialisationFunctions<T>,

    /// The priority of network messages for replicating `T`.
    pub message_priority: u32,

    #[doc(hidden)]
    pub phantom: PhantomData<T>,
}

impl<T: Component> Plugin for ComponentReplicationPlugin<T> {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<EntityReplicationPlugin>() {
            panic!("ComponentReplicationPlugin must be added after EntityReplicationPlugin");
        }

        app.insert_resource(ComponentSerialisationFunctions(self.serialisation.clone()));

        app.add_channel::<super::messages::ComponentReplicationChannel<T>>(ChannelConfiguration {
            reliable: ReliabilityGuarantee::Reliable,
            ordered: OrderingGuarantee::Sequenced,
            fragmented: true,
            priority: self.message_priority,
        });

        app.add_systems(PreUpdate, (
            super::systems::receive_component_messages::<T>
        ).in_set(PreUpdateReplicationSystems::UpdateComponents).chain());
    }
}

#[derive(Resource)]
pub(super) struct ComponentSerialisationFunctions<T: Component>(pub SerialisationFunctions<T>);