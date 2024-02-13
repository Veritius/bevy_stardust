
use std::sync::Arc;
use bevy::prelude::*;
use bevy_stardust::scheduling::{NetworkRead, NetworkWrite};
use quinn_proto::EndpointConfig;

/// Adds QUIC support to Stardust.
pub struct QuicTransportPlugin {
    /// How certificates should be verified for outgoing connections.
    pub authentication: TlsAuthentication,

    /// The number of reliable streams that are opened.
    /// Higher values reduce head of line blocking.
    pub reliable_streams: u32,

    /// The maximum duration of inactivity that is allowed before a connection is timed out, in seconds.
    /// Set this to something reasonable, like 30 seconds.
    pub timeout_delay: u32,
}

impl Plugin for QuicTransportPlugin {
    fn build(&self, app: &mut App) {
        // This step is a bit of a powerhouse
        app.add_systems(PreUpdate, (
            crate::incoming::quic_receive_packets_system,
            crate::polling::event_exchange_polling_system,
            crate::polling::connection_events_polling_system,
        ).chain().in_set(NetworkRead::Receive));

        app.add_systems(PostUpdate, crate::outgoing::quic_process_outgoing_system
            .in_set(NetworkWrite::Send));
        app.add_systems(Last, crate::logging::log_quic_events_system);

        app.init_resource::<crate::connections::ConnectionHandleMap>();
        app.insert_resource(PluginConfig {
            reliable_streams: self.reliable_streams,
            endpoint_config: Arc::new(EndpointConfig::default()),

            #[cfg(feature="dangerous")]
            server_cert_replacement: match &self.authentication {
                TlsAuthentication::Secure => None,
                TlsAuthentication::AlwaysVerify => Some(crate::crypto::dangerous::always_true_server_cert_verifier()),
                TlsAuthentication::Custom(verifier) => Some(verifier.clone()),
            },
        });
    }
}

/// How certificates should be authenticated.
/// By default, only `Secure` is available.
/// Set the `dangerous` feature flag for more options.
#[non_exhaustive]
#[derive(Debug, Default)]
pub enum TlsAuthentication {
    /// The certificate chain will be fully checked for authenticity.
    /// This is the safest option and what you should use for almost all games.
    #[default]
    Secure,

    /// The certificate is basically irrelevant and will always be verified.
    /// This is incredibly insecure and should only be used for testing.
    #[cfg(feature="dangerous")]
    AlwaysVerify,

    /// Use a custom implementation of `ServerCertVerifier`.
    #[cfg(feature="dangerous")]
    Custom(Arc<dyn rustls::client::ServerCertVerifier>),
}

/// Resource added by the plugin to store values defined/created when it was added.
#[derive(Resource)]
pub(crate) struct PluginConfig {
    pub reliable_streams: u32,
    pub endpoint_config: Arc<EndpointConfig>,

    #[cfg(feature="dangerous")]
    pub server_cert_replacement: Option<Arc<dyn rustls::client::ServerCertVerifier>>,
}