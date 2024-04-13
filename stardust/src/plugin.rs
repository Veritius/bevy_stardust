//! The Stardust core plugin.

use std::sync::Arc;
use bevy::prelude::*;
use crate::prelude::*;

/// The Stardust multiplayer plugin.
/// Adds the core functionality of Stardust, but does not add a transport layer.
pub struct StardustPlugin;

impl Plugin for StardustPlugin {
    fn build(&self, app: &mut App) {
        // Add ChannelRegistryMut
        app.insert_resource(ChannelRegistryMut(Box::new(ChannelRegistryInner::new())));

        // Add systems
        app.add_systems(Last, crate::connections::systems::despawn_closed_connections_system);
        app.add_systems(PostUpdate, (
            crate::messages::systems::clear_message_queue_system::<Outgoing>,
            crate::messages::systems::clear_message_queue_system::<Incoming>,
        ).in_set(NetworkWrite::Clear));

        // Setup orderings
        crate::scheduling::configure_scheduling(app);

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