//! The QUIC transport layer plugin.

use std::sync::Arc;
use bevy::prelude::*;

/// Adds a QUIC transport layer to the `App`.
pub struct QuicTransportPlugin {
    /// If enabled, the transport layer will permit acting as:
    /// - More than one client at once
    /// - More than one server at once
    /// - A client and a server at once
    /// 
    /// Most games do not need this functionality.
    /// If in doubt, set to `false`.
    pub allow_multipurpose: bool,

    /// Overrides the default QUIC transport configuration.
    /// This is for advanced users. If in doubt, set to `None`.
    pub transport_config_override: Option<Arc<quinn_proto::TransportConfig>>,

    /// Root certificates for connection authentication.
    /// If in doubt, you can use the `rustls-native-certs` or `webpki-roots` crates.
    pub root_certificates: rustls::RootCertStore,

    /// Chain of trust for connection authentication.
    pub certificate_chain: Vec<rustls::Certificate>,

    /// Server private key for encrypted messaging.
    pub private_key: rustls::PrivateKey,
}

impl Plugin for QuicTransportPlugin {
    fn build(&self, _app: &mut App) {
        todo!()
    }
}