use std::time::Duration;
use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_stardust::scheduling::*;
use crate::{appdata::{AppNetVersionWrapper, ApplicationNetworkVersion}, connection::PotentialNewPeer};

/// The UDP transport plugin.
pub struct UdpTransportPlugin {
    /// See the [`ApplicationNetworkVersion`] documentation.
    pub application_version: ApplicationNetworkVersion,

    /// The amount of reliable packet channels that are used.
    /// 
    /// Higher values reduce head-of-line blocking, but increase memory usage slightly.
    pub reliable_channel_count: u16,

    /// The length of the bitfield used to acknowledge packets.
    /// 
    /// Higher values improve packet loss detection.
    pub reliable_bitfield_length: u16,

    /// How long until a connection attempt will be abandoned due to no response.
    pub attempt_timeout: Duration,

    /// How long until a connection times out.
    pub connection_timeout: Duration,

    /// The length of a period of inactivity needed to send a 'keep-alive' packet, which maintains the connection.
    pub keep_alive_timeout: Duration,
}

/// Different default configurations to optimise the plugin for various things.
impl UdpTransportPlugin {
    /// Optimise configuration for balanced performance.
    pub fn balanced(application_version: ApplicationNetworkVersion) -> Self {
        Self {
            application_version,
            reliable_channel_count: 8,
            reliable_bitfield_length: 6,
            attempt_timeout: Duration::from_secs(10),
            connection_timeout: Duration::from_secs(20),
            keep_alive_timeout: Duration::from_secs(3),
        }
    }

    /// Optimise configuration for responsiveness. Useful for PvP shooters.
    pub fn responsive(application_version: ApplicationNetworkVersion) -> Self {
        // Use the balanced configuration as a baseline
        let mut config = Self::balanced(application_version);

        // Shooters don't send as many reliable packets.
        config.reliable_bitfield_length = 4;

        return config
    }

    /// Optimise configuration for efficiency. Useful for strategy games.
    #[inline]
    pub fn efficient(application_version: ApplicationNetworkVersion) -> Self {
        Self::balanced(application_version)
    }
}

impl Plugin for UdpTransportPlugin {
    fn build(&self, app: &mut App) {
        use crate::receiving::io_receiving_system;
        use crate::connection::{
            potential_new_peers_system,
            handshake_polling_system,
            close_connections_system,
        };
        use crate::sending::io_sending_system;

        // Send some warnings for potentially bad configuration
        if self.connection_timeout >= self.keep_alive_timeout {
            tracing::warn!("Connection timeout was greater than the keep-alive timeout: {}ms >= {}ms",
                self.connection_timeout.as_millis(), self.keep_alive_timeout.as_millis());
        }

        // Packet receiving system
        app.add_systems(PreUpdate, io_receiving_system
            .in_set(NetworkRead::Receive)
            .before(NetworkRead::Read));

        // State-specific packet reader systems
        app.add_systems(Update, (
            potential_new_peers_system,
            handshake_polling_system,
        ));

        // Packet transmitting systems
        app.add_systems(PostUpdate, (
            io_sending_system,
            close_connections_system,
        )
            .in_set(NetworkWrite::Send)
            .before(NetworkWrite::Clear));

        // Reset tick statistics at the end of the tick
        app.add_systems(Last, (
            crate::connection::statistics::reset_connection_statistics_system,
            crate::endpoint::statistics::reset_endpoint_statistics_system,
        ));

        app.add_event::<PotentialNewPeer>();

        // Add application context resource
        app.insert_resource(AppNetVersionWrapper(self.application_version.clone()));
    }
}