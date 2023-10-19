//! The Stardust core plugin.

use bevy::prelude::*;
use crate::prelude::*;
use crate::scheduling::*;
use crate::protocol::*;
use crate::channels::registry::ChannelRegistry;
use crate::channels::systems::*;

/// The Stardust multiplayer plugin.
/// Adds the core functionality of Stardust, but does not add a transport layer.
pub struct StardustPlugin;

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

        // Channel and hasher things
        app.insert_resource(ChannelRegistry::new());
        app.insert_resource(UniqueNetworkHasher::new());
        app.add_systems(PreStartup, complete_hasher);
    }
}