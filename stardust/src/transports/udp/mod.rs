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
pub struct UdpTransportPlugin;

impl Plugin for UdpTransportPlugin {
    fn build(&self, app: &mut App) {
        // Add states
        app.add_state::<UdpTransportState>();

        // Add systems
        app.add_systems(PostUpdate, apply_manager_action_system);
    }
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