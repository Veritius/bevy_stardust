mod shared;
use shared::*;

fn main() {
    let mut app = setup_app();

    // System to open the endpoint
    app.add_systems(Startup, |mut commands: Commands| {
        // Configuration for the endpoint
        let socket = UdpSocket::bind(SERVER_ADDRESS).unwrap();
        let config = Arc::new(EndpointConfig::default());

        // Create and spawn the endpoint
        let endpoint = QuicEndpoint::new(socket, config, None, true, None).unwrap();
        commands.spawn(endpoint);
    });

    app.run();
}