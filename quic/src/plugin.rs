
use std::sync::Arc;

use bevy::prelude::*;
use bevy_stardust::scheduling::{NetworkRead, NetworkWrite};
use quinn_proto::EndpointConfig;

/// Adds QUIC support to Stardust.
pub struct QuicTransportPlugin {
    /// Whether or not to allow self signed certificates.
    /// If set, this makes transport vulnerable to MITM attacks.
    pub allow_self_signed: bool,

    /// The number of reliable streams that are opened.
    /// Higher values reduce head of line blocking.
    pub reliable_streams: u32,

    /// The maximum duration of inactivity that is allowed before a connection is timed out, in seconds.
    /// Set this to something reasonable, like 30 seconds.
    pub timeout_delay: u32,
}

impl Plugin for QuicTransportPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, crate::incoming::quic_process_incoming_system
            .in_set(NetworkRead::Receive));
        app.add_systems(PostUpdate, crate::outgoing::quic_process_outgoing_system
            .in_set(NetworkWrite::Send));
        app.add_systems(Update, crate::logging::log_quic_events_system);

        app.insert_resource(PluginConfig {
            allow_self_signed: self.allow_self_signed,
            reliable_streams: self.reliable_streams,
            endpoint_config: Arc::new(EndpointConfig::default())
        });
    }
}

/// Resource added by the plugin to store values defined/created when it was added.
#[derive(Resource)]
pub(crate) struct PluginConfig {
    pub allow_self_signed: bool,
    pub reliable_streams: u32,
    pub endpoint_config: Arc<EndpointConfig>,
}