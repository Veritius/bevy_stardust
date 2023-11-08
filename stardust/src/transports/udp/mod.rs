//! Transport layer that operates over native UDP sockets.

#[cfg(target_arch = "wasm32")]
compile_error!("The UDP transport layer does not support wasm.");

mod manager;
mod connections;
mod ports;
mod sending;
mod receiving;
mod packet;
mod reliability;
mod ordering;

use std::ops::Range;
use bevy::prelude::*;
use crate::scheduling::*;
use self::{receiving::*, sending::*};
use manager::apply_manager_action_system;

// Expose manager
pub use manager::{UdpConnectionManager, startup_now};

/// This peer's transport layer version.
static COMPAT_THIS_VERSION: u32 = 0;
/// The versions of the transport layer that the transport layer will connect to.
static COMPAT_GOOD_VERSIONS: Range<u32> = 0..1;

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
        app.add_systems(TransportSendPackets, send_packets_system
            .run_if(not(in_state(UdpTransportState::Offline))));
    }
}

/// The current state of the transport layer.
/// Under no circumstances should you mutate this. Instead, use the [UdpConnectionManager] systemparam.
// TODO: Make it so state is still accessible but can't be mutated outside the transport layer
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Reflect, States)]
pub enum UdpTransportState {
    /// The transport layer is completely inactive.
    #[default]
    Offline,
    /// Ports are bound and there may be running connections.
    Active,
}

#[test]
fn compat_versions_check() {
    assert!(COMPAT_GOOD_VERSIONS.contains(&COMPAT_THIS_VERSION));
}