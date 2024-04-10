//! Main plugin for replication.

use std::marker::PhantomData;
use bevy::prelude::*;
use bevy_stardust::prelude::*;
use crate::*;
use crate::messaging::ReplicationData;

/// Adds functionality to support replication.
/// To replicate things, add other plugins:
/// - [`ReplicateResourcePlugin<T>`]
/// - [`ReplicateComponentPlugin<T>`]
/// 
/// This plugin must be added after [`StardustPlugin`].
pub struct ReplicationPlugin;

impl Plugin for ReplicationPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<StardustPlugin>() {
            panic!("StardustPlugin must be added before ReplicationPlugin");
        }

        app.register_type::<NetworkRoom>();
        app.register_type::<NetworkRoomMember>();

        todo!();
    }
}

/// Enables replicating the resource `T`.
/// 
/// This plugin must be added before [`StardustPlugin`].
/// Implicitly adds [`ReplicationPlugin`] if not present.
pub struct ReplicateResourcePlugin<T: ReplicableResource> {
    /// Message channel configuration.
    pub channel: ReplicationChannelConfiguration,

    #[doc(hidden)]
    pub phantom: PhantomData<T>,
}

impl<T: ReplicableResource> Plugin for ReplicateResourcePlugin<T> {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<ReplicationPlugin>() {
            app.add_plugins(ReplicationPlugin);
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
/// 
/// This plugin must be added before [`StardustPlugin`].
/// Implicitly adds [`ReplicationPlugin`] if not present.
pub struct ReplicateComponentPlugin<T: ReplicableComponent> {
    /// Message channel configuration.
    pub channel: ReplicationChannelConfiguration,

    #[doc(hidden)]
    pub phantom: PhantomData<T>,
}

impl<T: ReplicableComponent> Plugin for ReplicateComponentPlugin<T> {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<ReplicationPlugin>() {
            app.add_plugins(ReplicationPlugin);
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