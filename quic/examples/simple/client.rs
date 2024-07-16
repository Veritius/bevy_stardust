mod shared;

use bevy::prelude::*;
use bevy_stardust_quic::*;

fn main() {
    let mut app = shared::app();

    app.add_systems(Startup, |mut commands: Commands| {
        let endpoint = EndpointBuilder::client()
            .with_socket_addr("0.0.0.0:12345").unwrap()
            .with_protos(shared::APP_PROTOS)
            .with_trust_anchors(shared::trust_anchors())
            .build().unwrap();

        commands.spawn(endpoint);
    });

    app.run();
}