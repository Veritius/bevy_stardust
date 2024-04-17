use std::marker::PhantomData;
use bevy::prelude::*;
use bevy_stardust::prelude::*;
use crate::prelude::*;

/// Stardust channel for component replication for type `T`.
#[derive(Default)]
pub(crate) struct ComponentReplicationChannel<T: ReplicableComponent>(PhantomData<T>);

/// Enables replicating the component `T` on entities.
pub struct ComponentReplicationPlugin<T: ReplicableComponent> {
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

        app.add_channel::<ComponentReplicationChannel<T>>(ChannelConfiguration {
            reliable: ReliabilityGuarantee::Reliable,
            ordered: OrderingGuarantee::Sequenced,
            fragmented: true,
            priority: self.message_priority,
        });
    }
}