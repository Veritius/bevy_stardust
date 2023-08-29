//! Transport layer that operates over native UDP sockets.

mod client;
mod server;

mod peer;
mod ports;
mod listener;
mod attempt;
mod sending;
mod receiving;

use bevy::prelude::*;
use once_cell::sync::Lazy;
use semver::{Version, VersionReq};
use crate::{prelude::*, scheduling::*};
use self::{receiving::*, sending::*};

// Expose managers
pub use client::*;
pub use server::*;

static TRANSPORT_LAYER_VERSION: Lazy<Version> = Lazy::new(|| "0.2.0".parse::<Version>().unwrap());
static TRANSPORT_LAYER_REQUIRE: Lazy<VersionReq> = Lazy::new(|| "=0.2.0".parse::<VersionReq>().unwrap());

/// The UDP transport plugin. Use the systemparams ([UdpServerManager] and [UdpClientManager]) to set up connections.
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

        // Add reading systems
        app.add_systems(TransportReadPackets, udp_receive_packets_system_pooled
            .run_if(not(in_state(UdpTransportState::Disabled)))
            .run_if(processing_mode_is(ProcessingMode::Taskpool)));
        app.add_systems(TransportReadPackets, udp_receive_packets_system_single
            .run_if(not(in_state(UdpTransportState::Disabled)))
            .run_if(processing_mode_is(ProcessingMode::Single)));

        // Add writing systems
        app.add_systems(TransportSendPackets, udp_send_packets_system_pooled
            .run_if(not(in_state(UdpTransportState::Disabled)))
            .run_if(processing_mode_is(ProcessingMode::Taskpool)));
        app.add_systems(TransportSendPackets, udp_send_packets_system_single
            .run_if(not(in_state(UdpTransportState::Disabled)))
            .run_if(processing_mode_is(ProcessingMode::Single)));
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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Reflect, States)]
enum UdpTransportState {
    #[default]
    Disabled,
    Client,
    Server,
}

/// An error caused by an operation in the UDP transport layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UdpConnectionError {
    /// Some kind of IO-related error.
    /// This can be caused while binding to ports.
    IoError(std::io::ErrorKind),
    /// The transport layer was running in client mode when it should not have been.
    ClientExists,
    /// The transport layer was running in server mode when it should not have been.
    ServerExists,
    /// The transport layer was disconnected when it should not have been.
    Disconnected,
    /// The listen port was invalid.
    /// This can be caused by the listen port being in the active ports set.
    BadListenPort,
    /// The active ports set was empty.
    EmptyActivePorts,
}

fn processing_mode_is(target: ProcessingMode) -> impl Fn(Res<ProcessingMode>, Res<State<UdpTransportState>>) -> bool + Clone {
    move |mode: Res<ProcessingMode>, state: Res<State<UdpTransportState>>| -> bool {
        let resolved = match (*mode, state.get()) {
            (ProcessingMode::Best, UdpTransportState::Disabled) => ProcessingMode::Best,
            (ProcessingMode::Best, UdpTransportState::Client) => ProcessingMode::Single,
            (ProcessingMode::Best, UdpTransportState::Server) => ProcessingMode::Taskpool,
            (ProcessingMode::Single, _) => ProcessingMode::Single,
            (ProcessingMode::Taskpool, _) => ProcessingMode::Taskpool,
        };

        resolved == target
    }
}