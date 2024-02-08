use std::sync::Arc;
use rustls::pki_types::{TrustAnchor, CertificateDer, PrivateKeyDer};

/// Trust anchors for connection authentication.
pub struct TrustAnchors(pub(crate) Arc<TrustAnchorsInner>);

impl TrustAnchors {
    /// Creates a new TrustAnchors object from component parts.
    pub fn new(
        anchors: Vec<TrustAnchor<'static>>,
    ) -> Self {
        Self(Arc::new(TrustAnchorsInner::Owned(anchors)))
    }

    /// Use compiled-in trust anchors from the `webpki-roots` crate.
    #[cfg(feature="encryption-webpki-roots")]
    pub fn webpki() -> Self {
        Self(Arc::new(TrustAnchorsInner::Borrowed(webpki_roots::TLS_SERVER_ROOTS)))
    }
}

/// Trust anchors for connection authentication.
pub(crate) enum TrustAnchorsInner {
    Owned(Vec<TrustAnchor<'static>>),
    Borrowed(&'static [TrustAnchor<'static>])
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