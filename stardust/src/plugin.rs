//! The Stardust core plugin.

use bevy_app::prelude::*;
use crate::prelude::*;

/// The Stardust multiplayer plugin.
/// Adds the core functionality of Stardust, but does not add a transport layer.
pub struct StardustPlugin;

impl Plugin for StardustPlugin {
    fn build(&self, app: &mut App) {
        crate::channels::channel_build(app);

        // Add events
        app.add_event::<PeerDisconnectedEvent>();
        app.add_event::<PeerConnectedEvent>();

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