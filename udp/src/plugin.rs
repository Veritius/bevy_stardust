use bevy::prelude::*;
use bevy_stardust::{scheduling::{NetworkRead, NetworkWrite}, channels::registry::ChannelRegistry};
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

        app.insert_resource(UdpPluginConfig {
            reliable_pipes: self.reliable_pipes,
        });

        app.add_systems(PreUpdate, blocking_receive_packets_system
            .before(NetworkRead::Read)
            .in_set(NetworkRead::Receive));

        app.add_systems(PostUpdate, blocking_send_packets_system
            .before(NetworkWrite::Clear)
            .in_set(NetworkWrite::Send));
    }

    fn cleanup(&self, app: &mut App) {
        let channel_count = app.world.resource::<ChannelRegistry>().channel_count();
        if self.reliable_pipes != 0 && self.reliable_pipes as u32 > channel_count {
            // TODO: Deal with this problem without panicking
            panic!("The amount of reliable pipes ({}) exceeded the amount of channels ({})", self.reliable_pipes, channel_count);
        }
    }
}

#[derive(Resource)]
pub(crate) struct UdpPluginConfig {
    pub reliable_pipes: u8,
}