mod shared;

use bevy::prelude::*;
use bevy_stardust_quic::*;

const PRIVATE_KEY: &str = include_str!("../private_key.key");

fn main() {
    let mut app = shared::app();

    app.add_systems(Startup, |mut commands: Commands| {
        let endpoint = EndpointBuilder::server()
            .with_socket_addr("0.0.0.0:12345").unwrap()
            .with_protos(shared::app_protos())
            .with_trust_anchors(shared::trust_anchors())
            .with_credentials(Credentials::new(
                CertChain::from_iter(Some(Certificate::from_pem(shared::CERTIFICATE).unwrap())).unwrap(),
                PrivateKey::from_pem(PRIVATE_KEY).unwrap(),
            ).unwrap())
            .build().unwrap();

        commands.spawn(endpoint);
    });

    app.run();
}