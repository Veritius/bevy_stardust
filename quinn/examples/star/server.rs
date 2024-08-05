mod shared;

use std::net::UdpSocket;
use bevy::prelude::*;
use bevy_stardust_quinn::QuinnEndpoint;
use quinn::{rustls::pki_types::PrivateKeyDer, EndpointConfig, ServerConfig};
use rustls_pemfile::Item;

const PRIVATE_KEY: &str = include_str!("../certs/private_key.key");

fn private_key() -> PrivateKeyDer<'static> {
    let (item, _) = rustls_pemfile::read_one_from_slice(PRIVATE_KEY.as_bytes()).unwrap().unwrap();
    match item {
        Item::Pkcs8Key(key) => return key.into(),
        _ => panic!(),
    }
}

fn main() {
    let mut app = App::new();

    shared::setup(&mut app);

    app.add_systems(Startup, |mut commands: Commands| {
        let server_config = ServerConfig::with_single_cert(
            vec![shared::certificate()],
            private_key(),
        ).unwrap();

        let udp_socket = UdpSocket::bind(shared::SERVER_ADDRESS).unwrap();

        let endpoint = QuinnEndpoint::new(
            EndpointConfig::default(),
            Some(server_config),
            udp_socket,
            todo!(),
        ).unwrap();

        commands.spawn(endpoint);
    });

    app.run();
}