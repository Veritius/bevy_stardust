use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use bevy::prelude::*;
use bevy_stardust::prelude::*;
use bevy_stardust_quinn::QuinnPlugin;
use quinn::rustls::pki_types::CertificateDer;
use rustls_pemfile::Item;

pub const CERTIFICATE: &str = include_str!("../certs/certificate.crt");

pub const SERVER_ADDRESS: SocketAddr = SocketAddr::new(
    IpAddr::V4(Ipv4Addr::new(127, 0, 0, 0)),
    12345,
);

pub fn certificate() -> CertificateDer<'static> {
    let (item, _) = rustls_pemfile::read_one_from_slice(CERTIFICATE.as_bytes()).unwrap().unwrap();
    match item {
        Item::X509Certificate(cert) => return cert,
        _ => panic!(),
    }
}

pub fn setup(app: &mut App) {
    app.add_plugins((
        DefaultPlugins,
        StardustPlugin,
        QuinnPlugin,
    ));

    app.add_event::<MovementEvent>();

    app.add_channel::<MovementEvent>(ChannelConfiguration {
        consistency: ChannelConsistency::UnreliableSequenced,
        priority: 32,
    });
}

#[derive(Event)]
pub struct MovementEvent {
    pub direction: Vec2,
}