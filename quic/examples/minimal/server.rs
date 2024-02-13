mod shared;
use shared::*;

use std::sync::Arc;
use rustls::RootCertStore;
use bevy::prelude::*;
use bevy_stardust_quic::*;

fn main() {
    let mut server = setup_app();

    server.add_systems(Startup, |mut manager: QuicConnectionManager| {
        let certificate = certificate();
        let mut roots = RootCertStore::empty();
        roots.add(&certificate).unwrap();

        manager.open_server_endpoint(
            SERVER_ADDRESS,
            Arc::new(roots),
            vec![certificate],
            private_key(),
        ).unwrap();
    });

    server.run();
}