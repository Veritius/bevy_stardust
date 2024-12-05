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

    app.add_systems(Startup, |mut commands: Commands, runtime: Res<Runtime>| {
        let endpoint = EndpointBuilder::new()
            .with_runtime(runtime.handle())
            .bind(WILDCARD_ADDRESS).unwrap()
            .with_config(EndpointConfig::default())
            .client();

        let connection = Connection::connect(
            &runtime,
            &endpoint,
            ClientConfig::with_root_certificates(root_certs()).unwrap(),
            SERVER_ADDRESS,
            "server.example.com".into(),
        ).unwrap();

        commands.spawn(endpoint);
        commands.spawn(connection);
    });

    let started = Instant::now();
    app.add_systems(Update, move |mut events: EventWriter<AppExit>| {
        if started.elapsed() <= Duration::from_secs(10) { return }
        events.send(AppExit::Success);
    });

    app.run();
}