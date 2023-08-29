//! Transport layer that operates over native UDP sockets.

mod client;
mod server;

mod sending;
mod receiving;

use bevy::prelude::*;
use crate::{prelude::*, scheduling::*};
use self::{receiving::udp_receive_packets_system, sending::udp_send_packets_system};

pub use client::manager::*;
pub use server::manager::*;

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

        // Add systems
        app.add_systems(TransportReadPackets, udp_receive_packets_system
            .run_if(not(in_state(UdpTransportState::Disabled))));
        app.add_systems(TransportSendPackets, udp_send_packets_system
            .run_if(not(in_state(UdpTransportState::Disabled))));
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