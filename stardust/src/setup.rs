//! The Stardust core plugin.

use bevy::prelude::*;
use semver::Version;
use semver::VersionReq;

use crate::prelude::*;
use crate::scheduling::*;
use crate::protocol::*;
use crate::channels::registry::ChannelRegistry;
use crate::channels::systems::*;

/// The Stardust multiplayer plugin.
pub struct StardustPlugin {
    /// The version of your game. Used to prevent older/newer clients from joining.
    pub version: Version,
    /// The versions of the game this app can connect to.
    pub allows: VersionReq,
}

impl Plugin for StardustPlugin {
    fn build(&self, app: &mut App) {
        // Scheduling stuff
        add_schedules(app);
        app.add_systems(PreUpdate, network_pre_update);
        app.add_systems(PostUpdate, network_post_update);

        // Add events
        app.add_event::<DisconnectPeerEvent>();
        app.add_event::<PeerDisconnectedEvent>();
        app.add_event::<PeerConnectedEvent>();

        // Systems that check for things that shouldn't happen
        app.add_systems(PreUpdate, panic_on_channel_removal);

        // Systems for clearing the buffers
        app.add_systems(NetworkPreUpdateCleanup, clear_incoming_buffers_system);
        app.add_systems(NetworkPostUpdateCleanup, clear_outgoing_buffers_system);

        // Channel and hasher things
        app.insert_resource(ChannelRegistry::new());
        app.insert_resource(UniqueNetworkHasher::new());
        app.add_systems(PreStartup, complete_hasher);
    }
}