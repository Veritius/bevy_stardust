//! The Stardust core plugin.

use std::sync::Arc;
use bevy::prelude::*;
use crate::prelude::*;

/// The Stardust multiplayer plugin.
/// Adds the core functionality of Stardust, but does not add a transport layer.
pub struct StardustPlugin;

impl Plugin for StardustPlugin {
    fn build(&self, app: &mut App) {
        // Register connection types
        app.register_type::<NetworkPeer>();
        app.register_type::<NetworkPeerUid>();
        app.register_type::<NetworkGroup>();
        app.register_type::<NetworkPeerLifestage>();
        app.register_type::<NetworkSecurity>();
        app.register_type::<NetworkPerformanceReduction>();

        // Register channel types
        app.register_type::<ChannelId>();
        app.register_type::<ChannelConfiguration>();
        app.register_type::<ReliabilityGuarantee>();
        app.register_type::<OrderingGuarantee>();

        // Register messaging types
        app.register_type::<Direction>();
        app.register_type::<Incoming>();
        app.register_type::<Outgoing>();
        app.register_type::<NetworkMessages<Incoming>>();
        app.register_type::<NetworkMessages<Outgoing>>();

        // Setup orderings
        crate::scheduling::configure_scheduling(app);

        // Add ChannelRegistryMut
        app.insert_resource(ChannelRegistryMut(Box::new(ChannelRegistryInner::new())));

        // Add systems
        app.add_systems(Last, crate::connections::systems::despawn_closed_connections_system);
        app.add_systems(PostUpdate, (
            crate::messages::systems::clear_message_queue_system::<Outgoing>,
            crate::messages::systems::clear_message_queue_system::<Incoming>,
        ).in_set(NetworkWrite::Clear));

        // Hashing-related functionality
        #[cfg(feature="hashing")] {
            use crate::hashing::*;
            app.insert_resource(PendingHashValues::new());
            app.add_systems(PreStartup, finalise_hasher_system);    
        }
    }

    fn finish(&self, app: &mut App) {
        // Remove SetupChannelRegistry and put the inner into an Arc inside ChannelRegistry
        let registry = app.world.remove_resource::<ChannelRegistryMut>().unwrap();
        app.insert_resource(ChannelRegistry(Arc::from(registry.0)));
    }
}