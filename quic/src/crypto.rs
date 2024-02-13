#[cfg(feature="dangerous")]
pub mod dangerous {
    use std::sync::Arc;
    use rustls::client::ServerCertVerifier;

    pub fn always_true_server_cert_verifier() -> Arc<dyn ServerCertVerifier> {
        struct AlwaysTrueVerifier;

        impl ServerCertVerifier for AlwaysTrueVerifier {
            fn verify_server_cert(
                &self,
                end_entity: &rustls::Certificate,
                intermediates: &[rustls::Certificate],
                server_name: &rustls::ServerName,
                scts: &mut dyn Iterator<Item = &[u8]>,
                ocsp_response: &[u8],
                now: std::time::SystemTime,
            ) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
                Ok(rustls::client::ServerCertVerified::assertion())
            }
        }

        Arc::new(AlwaysTrueVerifier)
    }
}