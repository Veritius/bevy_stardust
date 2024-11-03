use std::sync::Arc;
use crate::config::*;
use crate::Connection;
use crate::Endpoint;
use crate::QuicManager;

impl Endpoint {
    /// Creates a new [`Endpoint`] component.
    pub fn new(
        certificate: CertificateChainOrigin,
        private_key: PrivateKeyOrigin,
        manager: &QuicManager,
    ) -> Self {
        todo!()
    }
}

impl Connection {
    /// Creates a new [`Connection`] component.
    pub fn new(
        root_certs: TrustAnchorStoreOrigin,
        server_name: Arc<str>,
        manager: &QuicManager,
    ) -> Self {
        todo!()
    }
}