
use std::sync::Arc;
use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_stardust::scheduling::{NetworkRead, NetworkWrite};
use quinn_proto::{EndpointConfig, TransportConfig, VarInt};

/// Adds QUIC support to Stardust.
pub struct QuicTransportPlugin {
    /// How certificates should be verified for outgoing connections.
    /// See the [`TlsAuthentication`] documentation for details.
    pub authentication: TlsAuthentication,

    /// The number of reliable streams that are opened.
    /// Higher values reduce head of line blocking.
    pub reliable_streams: u32,

    /// Overrides the `TransportConfig` used in connections.
    /// This is for advanced users - the defaults are good enough for almost all applications.
    pub transport_config_override: Option<Arc<quinn_proto::TransportConfig>>,
}

impl Plugin for QuicTransportPlugin {
    fn build(&self, app: &mut App) {
        // This step is a bit of a powerhouse
        app.add_systems(PreUpdate, (
            crate::receive::quic_receive_packets_system,
            crate::polling::event_exchange_polling_system,
            crate::polling::application_events_polling_system,
            crate::connections::despawn_drained_connections_system,
        ).chain().in_set(NetworkRead::Receive));

        app.add_systems(PostUpdate, (
            crate::outgoing::quic_process_outgoing_system,
            crate::connections::update_handle_map_system,
        ).chain().in_set(NetworkWrite::Send));

        app.add_systems(Last, crate::logging::log_quic_events_system);

        // Check if a transport config is provided, if not, just use defaults that are good for us
        let transport_config = if let Some(config) = self.transport_config_override.clone() {
            config
        } else {
            let mut config = TransportConfig::default();
            config.max_idle_timeout(Some(VarInt::from_u32(15_000).into())); // 15 seconds
            Arc::new(config)
        };

        // Add resources
        app.init_resource::<crate::connections::ConnectionHandleMap>();
        app.insert_resource(PluginConfig {
            reliable_streams: self.reliable_streams,
            transport_config,
            endpoint_config: Arc::new(EndpointConfig::default()),
            server_cert_verifier: match &self.authentication {
                TlsAuthentication::Secure => Arc::new(crate::crypto::WebPkiVerifier),

                #[cfg(feature="dangerous")]
                TlsAuthentication::AlwaysVerify => Arc::new(crate::crypto::dangerous::AlwaysTrueVerifier),

                #[cfg(feature="dangerous")]
                TlsAuthentication::Custom(verifier) => verifier.clone(),
            },
        });
    }
}

/// How certificates should be authenticated when using [`try_connect`](crate::endpoints::QuicConnectionManager::try_connect).
/// 
/// By default, only the `Secure` variant is available, providing the best security.
/// Set the `dangerous` feature flag for more options, including disabling authentication.
#[non_exhaustive]
#[derive(Debug, Default)]
pub enum TlsAuthentication {
    /// The certificate chain will be fully checked for authenticity.
    /// 
    /// This is the safest option and ensures the best security possible as long as your root CAs are good.
    #[default]
    Secure,

    /// The certificate provided by a remote connection will always be accepted as valid.
    /// 
    /// This completely invalidates all authentication and makes connections vulnerable to MITM attacks.
    /// This is useful if you don't care about TLS authentication or you're doing testing.
    #[cfg(feature="dangerous")]
    AlwaysVerify,

    /// Use a custom implementation of `ServerCertVerifier`.
    #[cfg(feature="dangerous")]
    Custom(Arc<dyn crate::crypto::ServerCertVerifier>),
}

/// Resource added by the plugin to store values defined/created when it was added.
#[derive(Resource)]
pub(crate) struct PluginConfig {
    pub reliable_streams: u32,
    pub transport_config: Arc<TransportConfig>,
    pub endpoint_config: Arc<EndpointConfig>,
    pub server_cert_verifier: Arc<dyn crate::crypto::ServerCertVerifier>,
}