use std::sync::Arc;
use bevy::prelude::*;
use anyhow::Result;
use rustls::{ServerConfig, ClientConfig, RootCertStore, pki_types::{CertificateDer, PrivateKeyDer}};

/// Configuration for TLS servers.
/// Can be changed at any time, but will only affect new connections.
#[derive(Resource)]
pub struct ServerTlsConfig(Arc<ServerConfig>);

impl ServerTlsConfig {
    /// Creates a new `ServerTlsConfig` from a certificate chain and private key.
    /// 
    /// Fails if `key_der` doesn't match `cert_chain`.
    pub fn with_single_cert(cert_chain: Vec<CertificateDer<'static>>, key_der: PrivateKeyDer<'static>) -> Result<Self> {
        let config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(cert_chain, key_der)?;

        Ok(Self(Arc::new(config)))
    }
}

/// Configuration for TLS clients.
/// Can be changed at any time, but will only affect new connections.
#[derive(Resource)]
pub struct ClientTlsConfig(Arc<ClientConfig>);

impl ClientTlsConfig {
    /// Use a custom set of root certificates as a trust anchor.
    /// 
    /// Most users shouldn't use this. Instead, you can use:
    /// - `with_native_roots` from the `encryption-native-roots` feature
    /// - `with_webpki_roots` from the `encryption-webpki-roots` feature
    pub fn with_custom_roots(roots: impl Into<Arc<RootCertStore>>) -> Self {
        let config = ClientConfig::builder()
            .with_root_certificates(roots)
            .with_no_client_auth();

        Self(Arc::new(config))
    }

    /// Uses native root certificates fetched from the OS as the trust anchor.
    /// Invalid certificates found in the OS will be silently ignored.
    /// 
    /// This can fail and is a very slow operation (use it in a future!)
    #[cfg(feature="encryption-native-roots")]
    pub fn with_native_roots() -> Result<Self> {
        let mut roots = RootCertStore::empty();
        roots.add_parsable_certificates(rustls_native_certs::load_native_certs()?);
        Ok(Self::with_custom_roots(roots))
    }

    /// Uses compiled-in root certificates from the [Common CA Database](https://www.ccadb.org/) run by Mozilla.
    #[cfg(feature="encryption-webpki-roots")]
    pub fn with_webpki_roots() -> Self {
        let mut roots = RootCertStore::empty();
        roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        Self::with_custom_roots(roots)
    }
}
