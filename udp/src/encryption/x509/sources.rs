//! Information related to authenticating X.509 certificates during a handshake.

pub use webpki::{types::{CertificateDer, TrustAnchor}, CertRevocationList, EndEntityCert};

use bevy::prelude::*;

/// End entity certificate. Used to authenticate we are who we say we are.
#[derive(Resource)]
pub struct EndEntityConfig {
    /// Sent when making a connection to verify we are who we say we are.
    pub end_entity_cert: EndEntityCert<'static>,
    /// Intermediate certificates between `end_entity_cert` and the root issuer.
    pub intermediates: Vec<CertificateDer<'static>>,
    /// The private key for `end_entity_cert`.
    pub private_key: Vec<u8>,
}

/// Trust anchors for connection authentication.
/// End-entity certificates from remote peers are checked against this.
#[derive(Resource)]
pub struct TrustAnchors(pub Vec<TrustAnchor<'static>>);

/// Revoked certificates, used when authenticating remote connections.
#[derive(Resource)]
pub struct CertificateRevocations(pub CertRevocationList<'static>);