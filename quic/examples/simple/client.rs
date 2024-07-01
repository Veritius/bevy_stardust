mod shared;
use quinn_proto::crypto::rustls::QuicClientConfig;
use rustls::{client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier}, crypto::CryptoProvider};
use shared::*;

fn main() {
    let mut app = setup_app();

    // System to open the endpoint
    app.add_systems(Startup, |mut commands: Commands| {
        // Install a default crypto provider
        CryptoProvider::install_default(rustls::crypto::ring::default_provider()).unwrap();

        // Configuration for the endpoint
        let socket = UdpSocket::bind(RANDOM_ADDRESS).unwrap();
        let config = Arc::new(EndpointConfig::default());

        let client_config = ClientConfig::new(Arc::new(QuicClientConfig::try_from(TlsClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(AlwaysVerify::new())
            .with_no_client_auth()
        ).unwrap()));

        // Create the endpoint, open a connection, then spawn
        let endpoint_id = commands.spawn_empty().id();
        let mut endpoint = QuicEndpoint::new(socket, config, None, true, None).unwrap();
        endpoint.connect(&mut commands, endpoint_id, client_config, SERVER_ADDRESS, "example.com").unwrap();
        commands.entity(endpoint_id).insert(endpoint);
    });

    app.run();
}

#[derive(Debug)]
struct AlwaysVerify(Arc<rustls::crypto::CryptoProvider>);

impl AlwaysVerify {
    fn new() -> Arc<Self> {
        Arc::new(Self(CryptoProvider::get_default().unwrap().clone()))
    }
}

impl ServerCertVerifier for AlwaysVerify {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls13_signature(message, cert, dss, &self.0.signature_verification_algorithms)
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls12_signature(message, cert, dss, &self.0.signature_verification_algorithms)
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        self.0.signature_verification_algorithms.supported_schemes()
    }
}