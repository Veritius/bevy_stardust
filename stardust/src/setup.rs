//! The Stardust core plugin.

use bevy::prelude::*;
use semver::Version;
use semver::VersionReq;

use crate::scheduling::*;
use crate::protocol::*;
use crate::channels::registry::ChannelRegistry;
use crate::channels::systems::*;

use crate::client::build_dedi_client;
use crate::server::build_dedi_server;
use crate::state::MultiplayerState;

/// The Stardust multiplayer plugin.
pub struct StardustPlugin {
    /// The version of your game. Used to prevent older/newer clients from joining.
    pub version: Version,
    /// The versions of the game this app can connect to.
    pub allows: VersionReq,
    /// How the multiplayer in your game operates.
    /// See the [MultiplayerMode] documentation for more.
    pub mode: MultiplayerMode,
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

        // Add some resources
        app.add_state::<MultiplayerState>();
        app.insert_resource(self.mode.clone());

        // Log mode choice
        info!("Stardust initialised as a {}", match self.mode {
            MultiplayerMode::DedicatedServer => "dedicated server",
            MultiplayerMode::DedicatedClient => "dedicated client",
            MultiplayerMode::ClientAndHost => "client and host",
            MultiplayerMode::ClientWithSingleplayer => "client with singleplayer",
            MultiplayerMode::ClientAndHostWithSingleplayer => "client and host with singleplayer",
        });

        // Add mode-specific functionality
        match self.mode {
            MultiplayerMode::DedicatedServer => build_dedi_server(app),
            MultiplayerMode::DedicatedClient => build_dedi_client(app),
            MultiplayerMode::ClientAndHost => todo!(),
            MultiplayerMode::ClientWithSingleplayer => todo!(),
            MultiplayerMode::ClientAndHostWithSingleplayer => todo!(),
        }
    }
}

/// How the multiplayer functionality in the game will work.
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
/// 
/// Note that having multiple transport layers and running in server mode is fine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Resource)]
pub enum MultiplayerMode {
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

impl MultiplayerMode {
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
