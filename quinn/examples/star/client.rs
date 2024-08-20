mod shared;

use std::sync::Arc;
use bevy::prelude::*;
use bevy_stardust_quinn::{Endpoints, RootCertStore};
use quinn_proto::{ClientConfig, EndpointConfig};

fn root_certs() -> Arc<RootCertStore> {
    let mut certs = RootCertStore::empty();
    certs.add(shared::certificate()).unwrap();
    return Arc::new(certs);
}

fn main() {
    let mut app = App::new();

    shared::setup(&mut app);

    app.add_systems(Startup, |mut endpoints: Endpoints| {
        endpoints.open(|builder| {
            builder.simple(
                Arc::new(EndpointConfig::default()),
                None,
                shared::WILDCARD_ADDRESS,
            ).unwrap().connect(|builder| {
                builder.simple(
                    ClientConfig::with_root_certificates(root_certs()).unwrap(),
                    shared::SERVER_ADDRESS,
                    "example.com".into()
                ).unwrap();
            })
        });
    });

    app.run();
}