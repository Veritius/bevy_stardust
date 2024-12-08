mod shared;

use std::{sync::Arc, time::{Duration, Instant}};
use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
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

    app.add_systems(Startup, |mut commands: Commands| {
        let endpoint = EndpointBuilder::new()
            .bind(SERVER_ADDRESS)
            .use_existing(Arc::new(EndpointConfig::default()))
            .client_only();

        // let connection = Connection::connect(
        //     &runtime,
        //     &endpoint,
        //     ClientConfig::with_root_certificates(root_certs()).unwrap(),
        //     SERVER_ADDRESS,
        //     "server.example.com".into(),
        // ).unwrap();

        // commands.spawn(connection);
    });

    let started = Instant::now();
    app.add_systems(Update, move |mut events: EventWriter<AppExit>| {
        if started.elapsed() <= Duration::from_secs(10) { return }
        events.send(AppExit::Success);
    });

    app.run();
}