use std::marker::PhantomData;
use bevy::prelude::*;
use bevy_stardust::prelude::*;
use crate::messaging::*;
use crate::plugin::*;
use crate::traits::*;

/// Trait for resources that can be replicated.
/// Automatically implemented for types that satisfy the requirements.
pub trait ReplicableResource: Resource + Replicable {}
impl<T> ReplicableResource for T where T: Resource + Replicable {}

/// Enables replicating the resource `T`.
pub struct ReplicateResourcePlugin<T: ReplicableResource> {
    /// Message channel configuration.
    pub channel: ReplicationChannelConfiguration,

    #[doc(hidden)]
    pub phantom: PhantomData<T>,
}

impl<T: ReplicableResource> Plugin for ReplicateResourcePlugin<T> {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<ReplicationPlugin>() {
            panic!("ReplicationPlugin must be added before ReplicateResourcePlugin")
        }

        app.add_channel::<ReplicationData<T>>(ChannelConfiguration {
            reliable: self.channel.reliable,
            ordered: OrderingGuarantee::Sequenced,
            fragmented: true,
            priority: self.channel.priority,
        });
    }
}