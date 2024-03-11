use std::time::Duration;
use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_stardust::scheduling::*;

/// The UDP transport plugin.
pub struct UdpTransportPlugin {
    /// The amount of reliable packet channels that are used.
    /// 
    /// Higher values reduce head-of-line blocking, but increase memory usage slightly.
    pub reliable_channel_count: u16,

    /// The length of a period of inactivity needed to send a 'keep-alive' packet, which maintains the connection.
    pub keep_alive_timeout: Duration,
}

impl Default for UdpTransportPlugin {
    fn default() -> Self {
        Self {
            reliable_channel_count: 8,
            keep_alive_timeout: Duration::from_secs(4),
        }
    }
}

impl Plugin for UdpTransportPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, crate::receiving::io_receiving_system
            .in_set(NetworkRead::Receive)
            .before(NetworkRead::Read));

        app.add_systems(PostUpdate, crate::sending::io_sending_system
            .in_set(NetworkWrite::Send)
            .before(NetworkWrite::Clear));
    }
}