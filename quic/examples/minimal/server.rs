mod shared;
use shared::*;

use std::sync::Arc;
use bevy_app::prelude::*;
use bevy_stardust_quic::*;

fn main() {
    let mut server = setup_app();

    server.add_systems(Startup, |mut manager: QuicConnectionManager| {
        manager.open_server_endpoint(
            SERVER_ADDRESS,
            Arc::new(root_cert_store()),
            vec![certificate()],
            private_key(),
        ).unwrap();
    });

    server.run();
}