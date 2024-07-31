mod shared;

use std::{net::UdpSocket, sync::Arc};
use bevy::prelude::*;
use bevy_stardust_quinn::QuinnEndpoint;
use quinn::{rustls::RootCertStore, ClientConfig, EndpointConfig};

fn roots() -> Arc<RootCertStore> {
    let mut store = RootCertStore::empty();
    store.add(shared::certificate()).unwrap();
    return Arc::new(store);
}

fn main() {
    let mut app = App::new();

    shared::setup(&mut app);

    app.add_systems(Startup, |mut commands: Commands| {
        let udp_socket = UdpSocket::bind(shared::SERVER_ADDRESS).unwrap();

        let endpoint = QuinnEndpoint::new(
            EndpointConfig::default(),
            None,
            udp_socket
        ).unwrap();

        let config = ClientConfig::with_root_certificates(roots()).unwrap();

        let connection = endpoint.connect(
            config,
            shared::SERVER_ADDRESS,
            "",
        ).unwrap();

        commands.spawn(endpoint);
        commands.spawn(connection);
    });

    app.run();
}