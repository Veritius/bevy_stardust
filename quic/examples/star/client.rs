mod shared;

use std::sync::Arc;
use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_stardust::prelude::*;
use bevy_stardust_quic::config::*;
use shared::*;

fn root_certs() -> Arc<RootCertStore> {
    let mut certs = RootCertStore::empty();
    certs.add(shared::certificate(CA_CERTIFICATE)).unwrap();
    return Arc::new(certs);
}

fn main() {
    let mut app = App::new();

    shared::setup(&mut app);

    app.add_systems(Startup, |mut endpoints: ResMut<Endpoints>, channels: Channels| {
        // let endpoint = EndpointBuilder::new()
        //     .bind(WILDCARD_ADDRESS)
        //     .with_config(Arc::new(EndpointConfig {
        //         quinn: Arc::new(quinn_proto::EndpointConfig::default()),
        //     }))
        //     .client_only();

        // endpoints.waiting.insert(endpoint);
    });

    app.add_systems(Update, |mut endpoints: ResMut<Endpoints>, mut commands: Commands| {
        while let Some(endpoint) = endpoints.waiting.poll() {
            let endpoint = endpoint.unwrap();

            // commands.spawn(Connection::connect(
            //     endpoint.clone(),
            //     SERVER_ADDRESS,
            //     "server.example.com".into(),
            //     ClientConfig {
            //         quinn: quinn_proto::ClientConfig::with_root_certificates(root_certs()).unwrap(),
            //     },
            // ));

            endpoints.endpoints.push(endpoint);
        }
    });

    app.run();
}