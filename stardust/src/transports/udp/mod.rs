//! Transport layer that operates over native UDP sockets.

#[cfg(target_arch = "wasm32")]
compile_error!("The UDP transport layer does not support wasm.");

mod manager;
mod peer;
mod ports;
mod sending;
mod receiving;
mod pending;

use bevy::prelude::*;
use once_cell::sync::Lazy;
use semver::{Version, VersionReq};
use crate::{prelude::*, scheduling::*};
use self::{receiving::*, sending::*};
use manager::apply_manager_action_system;

// Expose manager
pub use manager::UdpConnectionManager;

static TRANSPORT_LAYER_VERSION: Lazy<Version> = Lazy::new(|| TRANSPORT_LAYER_VERSION_STR.parse::<Version>().unwrap());
static TRANSPORT_LAYER_VERSION_STR: &str = "0.2.0";
static TRANSPORT_LAYER_REQUIRE: Lazy<VersionReq> = Lazy::new(|| TRANSPORT_LAYER_REQUIRE_STR.parse::<VersionReq>().unwrap());
static TRANSPORT_LAYER_REQUIRE_STR: &str = "=0.2.0";

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
        app.add_systems(TransportReadPackets, receive_packets_system
            .run_if(not(in_state(UdpTransportState::Offline))));
        // app.add_systems(TransportSendPackets, send_packets_system
        //     .run_if(not(in_state(UdpTransportState::Offline))));
    }
}

/// The current state of the transport layer.
/// Under no circumstances should you mutate this. Instead, use the [UdpConnectionManager] systemparam.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Reflect, States)]
pub enum UdpTransportState {
    /// Nothing going on. No ports are bound.
    #[default]
    Offline,
    /// Ports are bound and there may be running connections.
    Active,
}