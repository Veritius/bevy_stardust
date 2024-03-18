//! The Stardust core plugin.

use bevy_app::prelude::*;
use bevy_ecs::schedule::IntoSystemConfigs;
use crate::prelude::*;

/// The Stardust multiplayer plugin.
/// Adds the core functionality of Stardust, but does not add a transport layer.
pub struct StardustPlugin;

impl Plugin for StardustPlugin {
    fn build(&self, app: &mut App) {
        crate::channels::channel_build(app);

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
        crate::channels::channel_finish(app);
    }
}