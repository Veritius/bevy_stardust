//! Transport layer that operates over native UDP sockets.

mod client;
mod server;

mod sending;
mod receiving;

use bevy::prelude::*;
use crate::{prelude::*, scheduling::*};
use self::{receiving::udp_receive_packets_system, sending::udp_send_packets_system};

pub use client::UdpClientManager;
pub use server::UdpServerManager;

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
        app.add_state::<UdpTransportMode>();

        // Add resources
        app.insert_resource(self.mode.clone());

        // Add systems
        app.add_systems(TransportReadPackets, udp_receive_packets_system
            .run_if(not(in_state(UdpTransportMode::Disabled))));
        app.add_systems(TransportSendPackets, udp_send_packets_system
            .run_if(not(in_state(UdpTransportMode::Disabled))));
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
    /// This is suited to clients.
    Single,
    /// Runs all IO on multiple threads by breaking the load into tasks.
    /// This is suited to servers.
    Taskpool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Reflect, States)]
enum UdpTransportMode {
    #[default]
    Disabled,
    Client,
    Server,
}