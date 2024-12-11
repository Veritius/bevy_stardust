mod shared;

use std::sync::Arc;
use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_stardust::prelude::Channels;
use bevy_stardust_quic::*;
use shared::*;
use utilities::EndpointHandler;

// NOTE: It is very, very, very bad practice to compile-in private keys.
// This is only done for the sake of simplicity. In reality, you should
// get private keys and certificates from files.
const SERVER_CERTIFICATE: &str = include_str!("../certs/server.crt");
const SERVER_PRIVATE_KEY: &str = include_str!("../certs/server.key");

fn main() {
    let mut app = App::new();

    shared::setup(&mut app);

    app.init_resource::<EndpointHandler>();

    app.add_systems(Startup, |mut endpoints: ResMut<Endpoints>, channels: Channels| {
        // let endpoint = EndpointBuilder::new()
        //     .bind(SERVER_ADDRESS)
        //     .with_config(Arc::new(EndpointConfig {
        //         quinn: Arc::new(quinn_proto::EndpointConfig::default()),
        //     }))
        //     .with_config(Arc::new(ServerConfig {
        //         quinn: quinn_proto::ServerConfig::with_single_cert(
        //             vec![
        //                 shared::certificate(SERVER_CERTIFICATE),
        //             ],
        //             private_key(SERVER_PRIVATE_KEY),
        //         ).unwrap().into(),
        //         channels: channels.clone_arc(),
        //     }));

        // endpoints.waiting.insert(endpoint);
    });

    app.add_systems(Update, |mut endpoints: ResMut<Endpoints>, mut handler: ResMut<EndpointHandler>| {
        while let Some(endpoint) = endpoints.waiting.poll() {
            let endpoint = endpoint.unwrap();
            handler.insert(endpoint.clone().downgrade());
            endpoints.endpoints.push(endpoint);
        }
    });

    app.run();
}