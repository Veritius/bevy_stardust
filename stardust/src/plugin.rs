//! The Stardust core plugin.

use bevy::prelude::*;
use crate::prelude::*;
use crate::connections::*;
use crate::diagnostics::*;
use crate::messages::*;
use crate::streams::*;

/// The Stardust multiplayer plugin.
/// Adds the core functionality of Stardust, but does not add a transport layer.
pub struct StardustPlugin;

impl Plugin for StardustPlugin {
    fn name(&self) -> &str { "StardustPlugin" }

    fn build(&self, app: &mut App) {
        // Register connection types
        app.register_type::<Peer>();
        app.register_type::<PeerUid>();
        app.register_type::<PeerLifestage>();

        // Register diagnostic types
        app.register_type::<PeerStats>();
        app.register_type::<DropPackets>();
        app.register_type::<SimulateLatency>();

        // Register channel types
        app.register_type::<MessageConfiguration>();
        app.register_type::<MessageConsistency>();

        // Register messaging types
        app.register_type::<NetDirection>();
        app.register_type::<Incoming>();
        app.register_type::<Outgoing>();

        // Register events
        app.add_event::<DisconnectPeerEvent>();
        app.add_event::<PeerConnectingEvent>();
        app.add_event::<PeerConnectedEvent>();
        app.add_event::<PeerDisconnectingEvent>();
        app.add_event::<PeerDisconnectedEvent>();

        // Setup orderings
        crate::scheduling::configure_scheduling(app);

        // Setup channels
        crate::channels::build::<Messages>(app);
        crate::channels::build::<Streams>(app);

        // Add systems
        app.add_systems(PostUpdate, (
            crate::connections::clear_message_queues_system::<Outgoing>,
            crate::connections::clear_message_queues_system::<Incoming>,
        ).in_set(NetworkSend::Clear));
    }

    fn cleanup(&self, app: &mut App) {
        // Finish channels
        crate::channels::finish::<Messages>(app);
        crate::channels::finish::<Streams>(app);
    }
}