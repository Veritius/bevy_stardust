//! The Stardust core plugin.

use bevy::prelude::*;

use crate::scheduling::*;
use crate::protocol::*;
use crate::channels::registry::ChannelRegistry;
use crate::channels::systems::*;

use crate::client::build_dedi_client;
use crate::server::build_dedi_server;

/// The Stardust core networking plugin, with variants defining how the multiplayer will operate.
/// 
/// You can use the following table to identify what you should use.
/// 
/// | Can host | Can join | Can be singleplayer | Variant                       |
/// | -------- | -------- | ------------------- | ----------------------------- |
/// | Yes      | No       | No                  | DedicatedServer               |
/// | No       | Yes      | No                  | DedicatedClient               |
/// | Yes      | Yes      | No                  | ClientAndHost                 |
/// | No       | Yes      | Yes                 | ClientWithSingleplayer        |
/// | Yes      | Yes      | Yes                 | ClientAndHostWithSingleplayer |
pub enum StardustPlugin {
    /// Accepts remote peers and maintains an authoritative copy of the World.
    /// 
    /// Choose this if:
    /// - You want to run a dedicated/headless server.
    DedicatedServer,

    /// Connects to remote servers, but can never host its own server, and has no singleplayer.
    /// 
    /// Choose this if:
    /// - The client has no singleplayer or multiplayer code
    DedicatedClient,

    /// Can connect to remote servers, or host its own, but has no singleplayer.
    /// 
    /// Choose this if:
    /// - You want your game to connect to remote servers.
    /// - You want your game to be able to host a remote server and act as a client.
    /// - You don't have a singleplayer mode for your game.
    ClientAndHost,

    /// Can connect to remote servers, and can also run in a singleplayer state, but cannot host its own.
    /// 
    /// Choose this if:
    /// - You want your game to connect to remote servers.
    /// - You have a singleplayer mode in your game.
    ClientWithSingleplayer,

    /// Can host a server (acting as a client as well), connect to a server, or play singleplayer.
    /// 
    /// Choose this if:
    /// - You want your game to connect to remote servers.
    /// - You want your game to be able to host a remote server and act as a client.
    /// - You want your game to be able to run in singleplayer.
    ClientAndHostWithSingleplayer,
}

impl StardustPlugin {
    /// Chooses a multiplayer mode from the following values:
    /// - Can the app host servers?
    /// - Can the app join servers?
    /// - Can the app turn off multiplayer and run in singleplayer?
    /// 
    /// Values that don't make sense will panic.
    pub fn from_opts(can_host: bool, can_join: bool, can_be_singleplayer: bool) -> Self {
        match (can_host, can_join, can_be_singleplayer) {
            (true, false, false) => Self::DedicatedServer,
            (false, true, false) => Self::DedicatedClient,
            (true, true, false) => Self::ClientAndHost,
            (false, true, true) => Self::ClientWithSingleplayer,
            (true, true, true) => Self::ClientAndHostWithSingleplayer,
            (true, false, true) => panic!("Invalid value while making plugin: can't be a singleplayer host"),
            (false, false, true) => panic!("Invalid value while making plugin: can't be singleplayer only"),
            (false, false, false) => panic!("Invalid value while making plugin: can't be none of the above"),
        }
    }
}

impl Plugin for StardustPlugin {
    fn build(&self, app: &mut App) {
        // Scheduling stuff
        add_schedules(app);
        app.add_systems(PreUpdate, network_pre_update);
        app.add_systems(PostUpdate, network_post_update);

        // Systems that check for things that shouldn't happen
        app.add_systems(PreUpdate, panic_on_channel_removal);

        // Systems for clearing the buffers
        app.add_systems(NetworkPreUpdateCleanup, clear_incoming_buffers_system);
        app.add_systems(NetworkPostUpdateCleanup, clear_outgoing_buffers_system);

        // Channel and hasher things
        app.insert_resource(ChannelRegistry::new());
        app.insert_resource(UniqueNetworkHasher::new());
        app.add_systems(PreStartup, complete_hasher);

        info!("Stardust initialised as a {}", match self {
            Self::DedicatedServer => "dedicated server",
            Self::DedicatedClient => "dedicated client",
            Self::ClientAndHost => "client and host",
            Self::ClientWithSingleplayer => "client with singleplayer",
            Self::ClientAndHostWithSingleplayer => "client and host with singleplayer",
        });

        // Add mode-specific functionality
        match self {
            Self::DedicatedServer => build_dedi_server(app),
            Self::DedicatedClient => build_dedi_client(app),
            Self::ClientAndHost => todo!(),
            Self::ClientWithSingleplayer => todo!(),
            Self::ClientAndHostWithSingleplayer => todo!(),
        }
    }
}