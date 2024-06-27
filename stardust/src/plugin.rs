//! The Stardust core plugin.

use bevy::prelude::*;
use crate::prelude::*;
use crate::messages::channels;
use crate::diagnostics::NetworkPerformanceReduction;

/// The Stardust multiplayer plugin.
/// Adds the core functionality of Stardust, but does not add a transport layer.
pub struct StardustPlugin;

impl Plugin for StardustPlugin {
    fn build(&self, app: &mut App) {
        // Register connection types
        app.register_type::<Peer>();
        app.register_type::<PeerUid>();
        app.register_type::<PeerLifestage>();
        app.register_type::<NetworkSecurity>();
        app.register_type::<NetworkPerformanceReduction>();

        // Register channel types
        app.register_type::<ChannelId>();
        app.register_type::<channels::ChannelConfiguration>();
        app.register_type::<channels::ChannelConsistency>();

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
        channels::build_channels(app);

        // Add systems
        app.add_systems(Last, crate::connections::systems::despawn_closed_connections_system);
        app.add_systems(PostUpdate, (
            crate::connections::clear_message_queues_system::<Outgoing>,
            crate::connections::clear_message_queues_system::<Incoming>,
        ).in_set(NetworkSend::Clear));

        // Hashing-related functionality
        #[cfg(feature="hashing")] {
            use crate::hashing::*;
            app.insert_resource(PendingHashValues::new());
            app.add_systems(PreStartup, finalise_hasher_system);    
        }
    }

    fn finish(&self, app: &mut App) {
        // Log because of ordering stuff
        debug!("{} finished", std::any::type_name::<Self>());

        // Finish channels
        channels::finish_channels(app);
    }
}