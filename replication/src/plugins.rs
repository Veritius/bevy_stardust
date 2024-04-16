//! Main plugin for replication.

use bevy::app::PluginGroupBuilder;
use bevy::prelude::*;
use bevy_stardust::prelude::*;

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

        crate::scheduling::setup_schedules(app);
    }
}

/// Adds a set of plugins to replicate most Bevy components.
/// Adds [`RoomsPlugin`] - remove it if you don't want it!
pub struct ReplicationPlugins;

impl PluginGroup for ReplicationPlugins {
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