use bevy::prelude::*;
use bevy_stardust::scheduling::*;
use crate::{systems::*, sockets::socket_manager_system};

/// The UDP transport plugin. Adds the minimal functionality.
/// 
/// For almost all use cases, the `Default` implementation is good enough.
pub struct UdpTransportPlugin {
    /// The number of 'rivers' (what would be called channels in other crates) to use for reliable messaging.
    /// Higher values can perform better but increase the memory cost of a client in a linear fashion.
    /// Setting this to zero will disable reliability, sending all messages unreliably.
    pub river_count: u16,

    /// The number of bytes to use for reliability. Minimum of 1, maximum of 16.
    /// It's best to leave this on 4 unless you know what you're doing.
    pub bitfield_bytes: u8,
}

impl Default for UdpTransportPlugin {
    fn default() -> Self {
        Self {
            river_count: 8,
            bitfield_bytes: 4,
        }
    }
}

impl Plugin for UdpTransportPlugin {
    fn build(&self, app: &mut App) {
        use crate::config::*;

        // Make sure values are within acceptable ranges
        // Fields also have knock-on effects with eachother, so process that here
        let river_count = self.river_count.clamp(0, u16::MAX-2); // two rivers are reserved
        let bitfield_bytes = match river_count {
            0 => 0,
            _ => self.bitfield_bytes.clamp(1, 16),
        };

        // Add the config resource
        app.insert_resource(PluginConfig {
            river_count,
            bitfield_bytes,
        });

        // Add systems
        app.add_systems(PostUpdate, socket_manager_system);
        app.add_systems(PreUpdate, packet_listener_system
            .in_set(NetworkRead::Receive));
        app.add_systems(PostUpdate, packet_sender_system
            .in_set(NetworkWrite::Send));

        todo!();
    }
}