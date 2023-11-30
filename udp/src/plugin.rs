use bevy::prelude::*;
use bevy_stardust::scheduling::{NetworkRead, NetworkWrite};
use crate::{
    receiving::blocking_receive_packets_system,
    sending::blocking_send_packets_system
};

/// A transport layer for Stardust that uses native UDP sockets.
pub struct UdpTransportPlugin;

impl Plugin for UdpTransportPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, blocking_receive_packets_system
            .before(NetworkRead::Read)
            .in_set(NetworkRead::Receive));

        app.add_systems(PostUpdate, blocking_send_packets_system
            .before(NetworkWrite::Clear)
            .in_set(NetworkWrite::Send));
    }
}