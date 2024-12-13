mod shared;

use std::sync::Arc;
use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_stardust::prelude::*;
use bevy_stardust_quic::*;
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
        //     .with_channel_registry(channels.clone_arc())
        //     .use_existing(Arc::new(EndpointConfig::default()))
        //     .client_only();

        // endpoints.waiting.insert(endpoint);
    });

    app.add_systems(Update, |mut endpoints: ResMut<Endpoints>, mut commands: Commands| {
        // while let Some(endpoint) = endpoints.waiting.poll() {
        //     let endpoint = endpoint.unwrap();

        //     commands.spawn(Connection::connect(
        //         endpoint.clone(),
        //         SERVER_ADDRESS,
        //         "server.example.com".into(),
        //         ClientConfig::with_root_certificates(root_certs()).unwrap(),
        //     ));
        // }
    });

    app.run();
}