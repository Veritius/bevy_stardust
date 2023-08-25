//! The Stardust core plugin.

use bevy::prelude::*;

use crate::scheduling::*;
use crate::protocol::*;
use crate::channels::registry::ChannelRegistry;
use crate::channels::systems::*;

use crate::client::build_dedi_client;
use crate::server::build_dedi_server;

/// The Stardust core networking plugin.
pub enum Stardust {
    /// Connects remote clients and maintains an authoritative copy of the World.
    DedicatedServer,
    /// Connects to and is dependent on remote servers.
    DedicatedClient,
}

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
        app.insert_resource(match self {
            Self::DedicatedServer => PeerMode::DedicatedServer,
            Self::DedicatedClient => PeerMode::DedicatedClient,
        });

        // Add mode-specific functionality
        match self {
            Self::DedicatedServer => build_dedi_server(app),
            Self::DedicatedClient => build_dedi_client(app),
        }
    }
}

/// How this instance of the App is running.
#[derive(Debug, Resource, Clone, Copy, PartialEq, Eq)]
pub enum PeerMode {
    /// Connects remote clients and maintains an authoritative copy of the World.
    DedicatedServer,
    /// Connects to and is dependent on remote servers.
    DedicatedClient,
}