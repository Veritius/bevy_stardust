use std::sync::Arc;
use rustls::RootCertStore;

/// Verifies server certificates like [rustls's trait does](rustls::client::ServerCertVerifier),
/// but root certificates are given as part of the verification function.
pub trait ServerCertVerifier: Send + Sync {
    /// Verify a server certificate. See the [rustls docs](rustls::client::ServerCertVerifier::verify_server_cert) for a good explanation.
    /// Returning `()` means the certificate is validated and should be trusted.
    fn verify_server_cert(
        &self,
        end_entity: &rustls::Certificate,
        intermediates: &[rustls::Certificate],
        root_certs: Arc<RootCertStore>,
        server_name: &rustls::ServerName,
        scts: &mut dyn Iterator<Item = &[u8]>,
        ocsp_response: &[u8],
        now: std::time::SystemTime,
    ) -> Result<(), rustls::Error>;
}

impl std::fmt::Debug for dyn ServerCertVerifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "dyn ServerCertVerifier")
    }
}

/// Wraps a `ServerCertVerifier` from this crate and makes it acceptable for rustls to use.
pub(crate) struct ServerCertVerifierWrapper {
    pub roots: Arc<RootCertStore>,
    pub inner: Arc<dyn ServerCertVerifier>,
}

impl rustls::client::ServerCertVerifier for ServerCertVerifierWrapper {
    fn verify_server_cert(
        &self,
        end_entity: &rustls::Certificate,
        intermediates: &[rustls::Certificate],
        server_name: &rustls::ServerName,
        scts: &mut dyn Iterator<Item = &[u8]>,
        ocsp_response: &[u8],
        now: std::time::SystemTime,
    ) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
        let ver = self.inner.verify_server_cert(
            end_entity,
            intermediates,
            self.roots.clone(),
            server_name,
            scts,
            ocsp_response,
            now,
        );

        match ver {
            Ok(_) => Ok(rustls::client::ServerCertVerified::assertion()),
            Err(e) => Err(e),
        }
    }
}

/// Mostly identical to rustls' WebPkiVerifier but takes external root certificates.
/// Doesn't support certificate transparency.
pub(crate) struct WebPkiVerifier;

impl ServerCertVerifier for WebPkiVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &rustls::Certificate,
        intermediates: &[rustls::Certificate],
        root_certs: Arc<RootCertStore>,
        server_name: &rustls::ServerName,
        scts: &mut dyn Iterator<Item = &[u8]>,
        ocsp_response: &[u8],
        now: std::time::SystemTime,
    ) -> Result<(), rustls::Error> {
        // Parts of WebPkiVerifier are private, so we just make a new one. Not exactly very good, but it's cheap to do.
        let inner = rustls::client::WebPkiVerifier::new(root_certs.clone(), None);

        match rustls::client::ServerCertVerifier::verify_server_cert(
            &inner,
            end_entity,
            intermediates,
            server_name,
            scts,
            ocsp_response,
            now
        ) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
}

#[cfg(feature="insecure")]
pub mod insecure_verifiers {
    use std::sync::Arc;
    use super::ServerCertVerifier;

    pub(crate) struct AlwaysTrueVerifier;

    impl ServerCertVerifier for AlwaysTrueVerifier {
        fn verify_server_cert(
            &self,
            _end_entity: &rustls::Certificate,
            _intermediates: &[rustls::Certificate],
            _root_certs: Arc<rustls::RootCertStore>,
            _server_name: &rustls::ServerName,
            _scts: &mut dyn Iterator<Item = &[u8]>,
            _ocsp_response: &[u8],
            _now: std::time::SystemTime,
        ) -> Result<(), rustls::Error> {
            Ok(())
        }
    }
}