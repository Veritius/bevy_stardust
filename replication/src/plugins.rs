//! Main plugin for replication.

use std::marker::PhantomData;
use bevy::app::PluginGroupBuilder;
use bevy::prelude::*;
use bevy_stardust::prelude::*;
use crate::messages::ReplicationData;
use crate::prelude::*;

/// Adds functionality to support replication.
/// To replicate things, add other plugins:
/// - [`ReplicateResourcePlugin<T>`]
/// - [`ReplicateComponentPlugin<T>`]
/// 
/// This plugin must be added after [`StardustPlugin`].
pub struct CoreReplicationPlugin;

impl Plugin for CoreReplicationPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<StardustPlugin>() {
            panic!("StardustPlugin must be added before ReplicationPlugin");
        }

        app.register_type::<NetworkRoom>();

        crate::scheduling::setup_schedules(app);
    }
}

/// Enables replicating the resource `T`.
/// 
/// This plugin must be added before [`StardustPlugin`].
/// Implicitly adds [`ReplicationPlugin`] if not present.
pub struct ReplicateResourcePlugin<T: ReplicableResource> {
    /// If replication data should be sent reliably.
    pub reliability: ReliabilityGuarantee,

    /// The priority of the resource to replicate.
    /// Higher priority items will be replicated first.
    pub priority: u32,

    #[doc(hidden)]
    pub phantom: PhantomData<T>,
}

impl<T: ReplicableResource> Plugin for ReplicateResourcePlugin<T> {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<CoreReplicationPlugin>() {
            app.add_plugins(CoreReplicationPlugin);
        }

        app.add_channel::<ReplicationData<T>>(ChannelConfiguration {
            reliable: self.reliability,
            ordered: OrderingGuarantee::Sequenced,
            fragmented: true,
            priority: self.priority,
        });
    }
}

/// Enables replicating the component `T`.
/// 
/// This plugin must be added before [`StardustPlugin`].
/// Implicitly adds [`ReplicationPlugin`] if not present.
pub struct ReplicateComponentPlugin<T: ReplicableComponent> {
    /// If replication data should be sent reliably.
    pub reliability: ReliabilityGuarantee,

    /// The priority of the resource to replicate.
    /// Higher priority items will be replicated first.
    pub priority: u32,

    #[doc(hidden)]
    pub phantom: PhantomData<T>,
}

impl<T: ReplicableComponent> Plugin for ReplicateComponentPlugin<T> {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<CoreReplicationPlugin>() {
            app.add_plugins(CoreReplicationPlugin);
        }

        app.register_type::<ReplicateEntity>();
        app.register_type::<ReplicateDescendants>();      

        app.add_channel::<ReplicationData<T>>(ChannelConfiguration {
            reliable: self.reliability,
            ordered: OrderingGuarantee::Sequenced,
            fragmented: true,
            priority: self.priority,
        });
    }
}

/// Adds a set of plugins to replicate most Bevy components.
/// Adds [`RoomsPlugin`] - remove it if you don't want it!
pub struct DefaultPlugins;

impl PluginGroup for DefaultPlugins {
    fn build(self) -> PluginGroupBuilder {
        // const PRIORITY_HIGH: u32 = 128;
        // const PRIORITY_MED: u32 = 64;
        // const PRIORITY_LOW: u32 = 32;

        let group = PluginGroupBuilder::start::<Self>()
            .add(CoreReplicationPlugin)
            .add(crate::rooms::ReplicationRoomsPlugin);

        // #[cfg(feature="bevy_serialize")] {
        //     group = group
        //     .add(ReplicateComponentPlugin::<Name> {
        //         channel: ReplicationChannelConfiguration {
        //             reliable: ReliabilityGuarantee::Reliable,
        //             priority: PRIORITY_HIGH,
        //         },
        //         phantom: PhantomData,
        //     })
        //     .add(ReplicateComponentPlugin::<Transform> {
        //         channel: ReplicationChannelConfiguration {
        //             reliable: ReliabilityGuarantee::Unreliable,
        //             priority: PRIORITY_LOW,
        //         },
        //         phantom: PhantomData,
        //     });
        // }


        return group;
    }
}