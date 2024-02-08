use std::sync::Arc;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, CertificateRevocationListDer};

/// Trust anchors for connection authentication.
pub struct TrustAnchors(pub(crate) Arc<TrustAnchorsInner>);

impl TrustAnchors {
    /// Creates a new TrustAnchors object from component parts.
    pub fn new(
        anchors: Vec<CertificateDer<'static>>,
        revocations: CertificateRevocationListDer<'static>,
    ) -> Self {
        Self(Arc::new(TrustAnchorsInner {
            anchors,
            revocations,
        }))
    }
}

/// Trust anchors for connection authentication.
pub(crate) struct TrustAnchorsInner {
    pub anchors: Vec<CertificateDer<'static>>,
    pub revocations: CertificateRevocationListDer<'static>,
}

/// Certificate chain used to authenticate this peer.
pub struct AuthenticationSet(pub(crate) Arc<AuthenticationSetInner>);

impl AuthenticationSet {
    /// Creates a new AuthenticationSet from component parts.
    pub fn new(
        cert_chain: Vec<CertificateDer<'static>>,
        private_key: PrivateKeyDer<'static>,
    ) -> Self {
        Self(Arc::new(AuthenticationSetInner {
            cert_chain,
            private_key,
        }))
    }
}

pub(crate) struct AuthenticationSetInner {
    pub cert_chain: Vec<CertificateDer<'static>>,
    pub private_key: PrivateKeyDer<'static>,
}