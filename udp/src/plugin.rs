use bevy::prelude::*;
use bevy_stardust::scheduling::{NetworkRead, NetworkWrite};
use crate::{
    receiving::blocking_receive_packets_system,
    sending::blocking_send_packets_system
};

/// A transport layer for Stardust that uses native UDP sockets.
pub struct UdpTransportPlugin {
    /// How many 'pipes' reliability should be split over.
    /// This makes reliability more stable and 'reliable' but increases the memory usage of clients.
    /// The default is reasonable and you probably shouldn't have to go over that amount.
    /// 
    /// Maximum of `254`, defaults to `16`. Having a value of `0` disables reliability.
    pub reliable_pipes: u8,
}

impl Default for UdpTransportPlugin {
    /// Gives a rational default for most applications.
    fn default() -> Self {
        Self {
            reliable_pipes: 16,
        }
    }
}

impl Plugin for UdpTransportPlugin {
    fn build(&self, app: &mut App) {
        assert_ne!(self.reliable_pipes, u8::MAX, "The number of reliable pipes has a maximum of 254, not 255.");

        app.add_systems(PreUpdate, blocking_receive_packets_system
            .before(NetworkRead::Read)
            .in_set(NetworkRead::Receive));

        app.add_systems(PostUpdate, blocking_send_packets_system
            .before(NetworkWrite::Clear)
            .in_set(NetworkWrite::Send));
    }
}