mod shared;
use shared::*;

use std::sync::Arc;
use bevy_app::prelude::*;
use bevy_stardust_quic::*;

fn main() {
    let mut server = setup_app();

    server.add_systems(Startup, |mut manager: QuicConnectionManager| {
        let (certificate, private_key) = certificate_and_key();
        manager.open_server_endpoint(
            SERVER_ADDRESS,
            Arc::new(root_cert_store()),
            vec![certificate],
            private_key,
        ).unwrap();
    });

    server.run();
}

pub fn certificate_and_key() -> (Certificate, PrivateKey) {
    // Generates a self signed certificate
    let rcgen_cert = rcgen::generate_simple_self_signed(
            vec!["not.a.real.alt.name".to_string()]).unwrap();
    let rustls_cert = Certificate(rcgen_cert.serialize_der().unwrap());
    let rustls_key = PrivateKey(rcgen_cert.serialize_private_key_der());
    return (rustls_cert, rustls_key)
}