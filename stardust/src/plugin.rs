//! The Stardust core plugin.

use bevy::prelude::*;
use crate::messages::outgoing::clear_outgoing;
use crate::prelude::*;
use crate::protocol::*;
use crate::channels::registry::ChannelRegistry;
use crate::scheduling::NetworkWrite;

/// The Stardust multiplayer plugin.
/// Adds the core functionality of Stardust, but does not add a transport layer.
pub struct StardustPlugin;

impl Plugin for StardustPlugin {
    fn build(&self, app: &mut App) {
        // Add events
        app.add_event::<DisconnectPeerEvent>();
        app.add_event::<PeerDisconnectedEvent>();
        app.add_event::<PeerConnectedEvent>();

        // Channel and hasher things
        app.insert_resource(ChannelRegistry::new());
        app.insert_resource(ProtocolIdHasher::new());
        app.add_systems(PreStartup, complete_hasher);

        app.add_systems(PostUpdate, clear_outgoing
            .in_set(NetworkWrite::Clear)
            .after(NetworkWrite::Send));
    }
}