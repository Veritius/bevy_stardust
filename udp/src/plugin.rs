use std::time::Duration;
use bevy_app::prelude::*;

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
        todo!();
    }
}