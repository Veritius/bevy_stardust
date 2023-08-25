//! The `Stardust` plugin.

use bevy::prelude::*;

use crate::scheduling::*;
use crate::protocol::*;
use crate::channels::registry::ChannelRegistry;
use crate::channels::systems::*;

use crate::client::build_dedi_client;
use crate::server::build_dedi_server;

/// The Stardust networking plugin.
pub struct Stardust(pub NetworkMode);

impl Plugin for Stardust {
    fn build(&self, app: &mut App) {
        // Scheduling stuff
        add_schedules(app);
        app.add_systems(PreUpdate, network_pre_update);
        app.add_systems(PostUpdate, network_post_update);

        // Some systems to check stuff
        app.add_systems(PreUpdate, panic_on_channel_removal);

        // Systems for clearing the buffers
        app.add_systems(NetworkPreUpdateCleanup, clear_incoming_buffers_system);
        app.add_systems(NetworkPostUpdateCleanup, clear_outgoing_buffers_system);

        // Channel and hasher things
        app.insert_resource(ChannelRegistry::new());
        app.insert_resource(UniqueNetworkHasher::new());
        app.add_systems(PreStartup, complete_hasher);

        // Insert network mode resource
        app.insert_resource(self.0.clone());

        // Add mode-specific functionality
        match self.0 {
            NetworkMode::DedicatedServer => build_dedi_server(app),
            NetworkMode::DedicatedClient => build_dedi_client(app),
        }
    }
}

/// Whether this App is a server or a client.
#[derive(Debug, Resource, Clone, Copy, PartialEq, Eq)]
pub enum NetworkMode {
    /// Connects remote clients and maintains an 'authoritative' view of the world.
    DedicatedServer,
    /// Connects to remote servers and is dependent on them to function.
    DedicatedClient,
}