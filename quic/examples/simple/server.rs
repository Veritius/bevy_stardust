mod shared;
use shared::*;

const CERTIFICATE: &str = include_str!("server.crt");
const PRIVATE_KEY: &str = include_str!("server.key");

fn main() {
    let mut app = setup_app();

    // System to open the endpoint
    app.add_systems(Startup, |mut commands: Commands| {
        // Configuration for the endpoint
        let socket = UdpSocket::bind(SERVER_ADDRESS).unwrap();
        let config = Arc::new(EndpointConfig::default());

        // Setup the server config, including TLS stuff
        let certificate = rustls_pemfile::certs(&mut CERTIFICATE.as_bytes()).map(|result| result.unwrap()).collect();
        let private_key = rustls_pemfile::private_key(&mut PRIVATE_KEY.as_bytes()).unwrap().unwrap();
        let server_config = Some(Arc::new(ServerConfig::with_single_cert(certificate, private_key).unwrap()));

        // Create and spawn the endpoint
        let endpoint = QuicEndpoint::new(socket, config, server_config, true, None).unwrap();
        commands.spawn(endpoint);
    });

    app.run();
}