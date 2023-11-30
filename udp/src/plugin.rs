use bevy::prelude::*;
use bevy_stardust::scheduling::{NetworkRead, NetworkWrite};
use crate::{
    receiving::blocking_receive_packets_system,
    sending::deferred_send_packets_system
};

/// A transport layer for Stardust that uses native UDP sockets.
pub struct UdpTransportPlugin {
    /// How the sending system's I/O should be run.
    /// The receiving system will always run in `Blocking` mode.
    pub send_mode: SystemIoMode,
}

impl Plugin for UdpTransportPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, blocking_receive_packets_system
            .before(NetworkRead::Read)
            .in_set(NetworkRead::Receive));

        match self.send_mode {
            SystemIoMode::Deferred => {
                app.add_systems(PostUpdate, deferred_send_packets_system
                    .before(NetworkWrite::Clear)
                    .in_set(NetworkWrite::Send));
            },
            SystemIoMode::Blocking => todo!(),
        }
    }
}

/// Parallelism config for I/O systems in the plugin.
#[derive(Debug, Default)]
pub enum SystemIoMode {
    /// Defers I/O operations to Bevy to occur at some point.
    #[default]
    Deferred,

    /// Runs as an exclusive system, using all CPU cores to perform I/O until finished.
    /// This may result in Bevy tasks (compute, io) being CPU deprived until the system finishes.
    Blocking,
}