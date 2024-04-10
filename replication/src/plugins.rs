//! Main plugin for replication.

use std::marker::PhantomData;
use bevy::prelude::*;
use bevy_stardust::prelude::*;
use crate::*;
use crate::messaging::ReplicationData;

/// Adds basic replication functionality.
/// 
/// This must be added:
/// - After the Stardust plugin
/// - Before other replication plugins
pub struct ReplicationPlugin;

impl Plugin for ReplicationPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<StardustPlugin>() {
            panic!("StardustPlugin must be added before ReplicationPlugin")
        }

        todo!();
    }
}

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

/// Enables replicating the component `T`.
pub struct ReplicateComponentPlugin<T: ReplicableComponent> {
    /// Message channel configuration.
    pub channel: ReplicationChannelConfiguration,

    #[doc(hidden)]
    pub phantom: PhantomData<T>,
}

impl<T: ReplicableComponent> Plugin for ReplicateComponentPlugin<T> {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<ReplicationPlugin>() {
            panic!("ReplicationPlugin must be added before ReplicateComponentPlugin")
        }

        app.register_type::<ReplicateEntity>();
        app.register_type::<ReplicateDescendants>();      

        app.add_channel::<ReplicationData<T>>(ChannelConfiguration {
            reliable: self.channel.reliable,
            ordered: OrderingGuarantee::Sequenced,
            fragmented: true,
            priority: self.channel.priority,
        });
    }
}