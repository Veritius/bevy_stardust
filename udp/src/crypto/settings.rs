use std::{sync::Arc, path::Path};
use bevy::prelude::*;
use anyhow::{Result, bail};
use rustls::{ServerConfig, ClientConfig, RootCertStore, pki_types::{CertificateDer, PrivateKeyDer}};

/// Configuration for TLS servers.
/// Can be changed at any time, but will only affect new connections.
#[derive(Resource)]
pub struct ServerTlsConfig(Arc<ServerConfig>);

impl ServerTlsConfig {
    /// Clones the internal `Arc` used to store the server config data.
    pub fn clone_arc(&self) -> Arc<ServerConfig> {
        self.0.clone()
    }

    /// Creates a new `ServerTlsConfig` from a certificate chain and private key.
    /// 
    /// Fails if `key_der` doesn't match `cert_chain`.
    pub fn with_single_cert(cert_chain: Vec<CertificateDer<'static>>, key_der: PrivateKeyDer<'static>) -> Result<Self> {
        let mut config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(cert_chain, key_der)?;

        // no secret extraction for you
        config.enable_secret_extraction = false;

        Ok(Self(Arc::new(config)))
    }

    /// Creates a new `ServerTlsConfig` from a certificate chain and private key found in the given paths.
    /// Uses the first private key found. Invalid certificates or keys found in files will be silently ignored.
    /// 
    /// Fails if the files can't be accessed, are invalid, or if the private key doesn't match the certificate chain.
    pub fn with_single_cert_from_file(cert_chain_path: &Path, key_der_path: &Path) -> Result<Self> {
        use std::{io::BufReader, fs::File};

        // Get all certs
        let mut cert_reader = BufReader::new(File::open(cert_chain_path)?);
        let cert_chain = rustls_pemfile::certs(&mut cert_reader).filter_map(|f| f.ok()).collect();

        // Get all keys
        let mut key_reader = BufReader::new(File::open(key_der_path)?);
        let key_der = match rustls_pemfile::private_key(&mut key_reader)? {
            Some(value) => value,
            None => bail!("No keys found in {key_der_path:?}"),
        };

        Self::with_single_cert(cert_chain, key_der)
    }
}

/// Configuration for TLS clients.
/// Can be changed at any time, but will only affect new connections.
#[derive(Resource)]
pub struct ClientTlsConfig(Arc<ClientConfig>);

impl ClientTlsConfig {
    /// Clones the internal `Arc` used to store the client config data.
    pub fn clone_arc(&self) -> Arc<ClientConfig> {
        self.0.clone()
    }

    /// Use a custom set of root certificates as a trust anchor.
    /// 
    /// Most users shouldn't use this. Instead, you can use:
    /// - `with_native_roots` from the `encryption-native-roots` feature
    /// - `with_webpki_roots` from the `encryption-webpki-roots` feature
    pub fn with_custom_roots(roots: impl Into<Arc<RootCertStore>>) -> Self {
        let mut config = ClientConfig::builder()
            .with_root_certificates(roots)
            .with_no_client_auth();

        // no secret extraction for you either
        config.enable_secret_extraction = false;

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
