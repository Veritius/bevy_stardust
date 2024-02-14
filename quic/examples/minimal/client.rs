mod shared;
use shared::*;

use std::sync::Arc;
use bevy::prelude::*;
use bevy_stardust_quic::*;

fn main() {
    let mut client = setup_app();

    client.add_systems(Startup, |mut manager: QuicConnectionManager| {
        manager.open_client_endpoint(
            CLIENT_ADDRESS,
            Arc::new(root_cert_store()),
        ).unwrap();
    });

    client.add_systems(PostStartup, |endpoints: Query<Entity, With<QuicEndpoint>>, mut manager: QuicConnectionManager| {
        manager.try_connect(
            endpoints.single(),
            SERVER_ADDRESS,
            SERVER_ALT_NAME
        ).unwrap();
    });

    client.run();
}