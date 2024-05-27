use std::time::Duration;
use bevy::prelude::*;
use bevy_stardust::prelude::*;
use crate::{appdata::ApplicationNetworkVersion, connection::PotentialNewPeer, schedule::*};

/// The UDP transport plugin.
/// 
/// This must be added to the App *after* all channels are registered.
pub struct UdpTransportPlugin {
    /// See the [`ApplicationNetworkVersion`] documentation.
    pub application_version: ApplicationNetworkVersion,

    /// The amount of reliable packet channels that are used.
    /// 
    /// Higher values reduce head-of-line blocking, but increase memory usage slightly.
    pub reliable_channel_count: u8,

    /// The length of the bitfield used to acknowledge packets.
    /// Must be within the range of `1` to `16` inclusive.
    /// 
    /// Higher values improve packet loss detection.
    pub reliable_bitfield_length: u16,

    /// How long until a connection attempt will be abandoned due to no response.
    pub attempt_timeout: Duration,

    /// How long until a connection times out.
    pub connection_timeout: Duration,

    /// The length of a period of inactivity needed to send a 'keep-alive' packet, which maintains the connection.
    pub keep_alive_timeout: Duration,

    /// The amount of time to wait for a closing message when a peer is in the `Closing` state before doing it anyway.
    pub close_wait_time: Duration,
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
            close_wait_time: Duration::from_secs(30),
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
        // Check if the Stardust plugin is added
        if !app.is_plugin_added::<StardustPlugin>() {
            panic!("StardustPlugin muaest be added before UdpTransportPlugin");
        }

        // Send some warnings for potentially bad configuration
        if self.keep_alive_timeout >= self.connection_timeout {
            tracing::warn!("Keep-alive timeout was greater than the connection timeout: {}ms >= {}ms",
                self.keep_alive_timeout.as_millis(), self.connection_timeout.as_millis());
        }

        // Sanity checks for plugin configuration
        // Most of these, if not checked here, would panic elsewhere
        assert!(self.reliable_bitfield_length > 0,
            "The length of reliable bitfields must be above 0");
        assert!(self.reliable_bitfield_length < 16,
            "The length of reliable bitfields cannot exceed 16");

        app.configure_sets(PreUpdate, PreUpdateSet::PacketRead
            .before(PreUpdateSet::TickEstablished)
            .before(PreUpdateSet::HandleUnknown)
        );

        app.configure_sets(PostUpdate, PostUpdateSet::PacketSend
            .after(PostUpdateSet::FramePacking)
            .after(PostUpdateSet::HandshakeSend)
            .before(PostUpdateSet::CloseEndpoints)
            .before(PostUpdateSet::CloseConnections)
            .before(PostUpdateSet::UpdateStatistics)
        );

        app.add_systems(PreUpdate, crate::receiving::io_receiving_system
            .in_set(PreUpdateSet::PacketRead));

        app.add_systems(PostUpdate, crate::sending::io_sending_system
            .in_set(PostUpdateSet::PacketSend));

        app.add_systems(PostUpdate, crate::endpoint::close_endpoints_system
            .in_set(PostUpdateSet::CloseEndpoints));

        app.add_event::<PotentialNewPeer>();

        // Add application context resource
        app.insert_resource(PluginConfiguration {
            application_version: self.application_version.clone(),
            reliable_bitfield_length: self.reliable_bitfield_length as usize,
            attempt_timeout: self.attempt_timeout,
            connection_timeout: self.connection_timeout,
            keep_alive_timeout: self.keep_alive_timeout,
        });
    }
}

#[derive(Resource)]
pub(crate) struct PluginConfiguration {
    pub application_version: ApplicationNetworkVersion,
    pub reliable_bitfield_length: usize,
    pub attempt_timeout: Duration,
    pub connection_timeout: Duration,
    pub keep_alive_timeout: Duration,
}