mod shared;

use std::sync::Arc;
use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_stardust_quinn::*;
use quinn_proto::{ClientConfig, EndpointConfig};
use shared::*;

fn root_certs() -> Arc<RootCertStore> {
    let mut certs = RootCertStore::empty();
    certs.add(shared::certificate(CA_CERTIFICATE)).unwrap();
    return Arc::new(certs);
}

fn main() {
    let mut app = App::new();

    shared::setup(&mut app);

    app.add_systems(Startup, |mut commands: Commands| {
        commands.spawn_empty().make_endpoint(MakeEndpoint {
            socket: QuicSocket::new(WILDCARD_ADDRESS).unwrap(),
            config: Arc::new(EndpointConfig::default()),
            server: None,
        }).connect(OpenConnection {
            remote: SERVER_ADDRESS,
            config: ClientConfig::with_root_certificates(root_certs()).unwrap(),
            server_name: "example.com".into(),
        });
    });

    app.run();
}