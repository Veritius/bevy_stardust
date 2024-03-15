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

    /// The length of a period of inactivity needed to send a 'keep-alive' packet, which maintains the connection.
    pub keep_alive_timeout: Duration,
}

impl UdpTransportPlugin {
    /// Optimised for balanced performance.
    pub fn balanced(application_version: ApplicationNetworkVersion) -> Self {
        Self {
            application_version,
            reliable_channel_count: 8,
            keep_alive_timeout: Duration::from_secs(4),
        }
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