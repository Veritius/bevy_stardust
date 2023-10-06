//! Transport layer that operates over native UDP sockets.

mod manager;
mod peer;
mod ports;
mod sending;
mod receiving;

use bevy::prelude::*;
use once_cell::sync::Lazy;
use semver::{Version, VersionReq};
use crate::{prelude::*, scheduling::*};
use self::{receiving::*, sending::*};

// Expose manager
pub use manager::*;

static TRANSPORT_LAYER_VERSION: Lazy<Version> = Lazy::new(|| "0.2.0".parse::<Version>().unwrap());
static TRANSPORT_LAYER_REQUIRE: Lazy<VersionReq> = Lazy::new(|| "=0.2.0".parse::<VersionReq>().unwrap());
const PACKET_HEADER_SIZE: usize = 5;
const PACKET_MAX_BYTES: usize = 1472;

/// The UDP transport plugin. Use the [UdpConnectionManager] systemparam to set up connections while in a system.
#[derive(Debug)]
pub struct UdpTransportPlugin {
    /// How the transport layer should process IO. See [ProcessingMode's documentation](ProcessingMode) for more.
    pub mode: ProcessingMode,
}

impl UdpTransportPlugin {
    /// Configures the plugin to be good enough for most uses.
    pub fn best_guess() -> Self {
        Self {
            mode: ProcessingMode::Best,
        }
    }

    /// Configures the plugin to perform well with a single peer.
    pub fn single() -> Self {
        Self {
            mode: ProcessingMode::Single,
        }
    }

    /// Configures the plugin to work well with multiple peers.
    pub fn scalable() -> Self {
        Self {
            mode: ProcessingMode::Taskpool,
        }
    }
}

impl Plugin for UdpTransportPlugin {
    fn build(&self, app: &mut App) {
        // Add states
        app.add_state::<UdpTransportState>();

        // Add resources
        app.insert_resource(self.mode.clone());
    }
}

/// How the UDP transport layer will utilise threading.
#[derive(Debug, Default, Resource, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum ProcessingMode {
    /// Picks the best processing mode for you.
    /// This will (likely) be `Single` on clients and `Taskpool` on servers.
    #[default]
    Best,
    /// Runs all IO on a single thread.
    /// This is best suited to clients.
    Single,
    /// Runs all IO on multiple threads by breaking the load into tasks.
    /// This is best suited to servers.
    Taskpool,
}

/// The current state of the transport layer.
/// Under no circumstances should you mutate this. Instead, use the [UdpConnectionManager] systemparam.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Reflect, States)]
pub enum UdpTransportState {
    /// Nothing going on.
    #[default]
    Offline,
    /// Standing by, with ports allocated, but no active connection.
    Standby,
    /// Running as a client and connected to a server.
    Client,
    /// Running as a server and listening for connections.
    Server,
}