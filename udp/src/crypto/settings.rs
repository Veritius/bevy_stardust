use std::{sync::Arc, path::Path};
use bevy::prelude::*;
use anyhow::{Result, bail};
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

    /// Creates a new `ServerTlsConfig` from a certificate chain and private key found in the given paths.
    /// Invalid certificates or keys found in files will be silently ignored.
    /// 
    /// Fails if the files can't be accessed, are invalid, or if the private key doesn't match the certificate chain.
    pub fn with_single_cert_from_file(cert_chain_path: &Path, key_der_path: &Path) -> Result<Self> {
        use std::{io::BufReader, fs::File};

        // Get all certs
        let mut cert_reader = BufReader::new(File::open(cert_chain_path)?);
        let cert_chain = rustls_pemfile::certs(&mut cert_reader).filter_map(|f| f.ok()).collect();

        // Get all keys
        let mut key_reader = BufReader::new(File::open(key_der_path)?);
        let key_der = loop {
            use rustls_pemfile::Item;
            match rustls_pemfile::read_one(&mut key_reader)? {
                Some(Item::Pkcs1Key(key)) => break key.into(),
                Some(Item::Pkcs8Key(key)) => break key.into(),
                Some(Item::Sec1Key(key)) => break key.into(),
                Some(_) => continue,
                None => bail!("No keys found in path {key_der_path:?}"),
            }
        };

        Self::with_single_cert(cert_chain, key_der)
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
