mod shared;

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_stardust::prelude::Channels;
use bevy_stardust_quic::*;
use shared::*;

// NOTE: It is very, very, very bad practice to compile-in private keys.
// This is only done for the sake of simplicity. In reality, you should
// get private keys and certificates from files.
const SERVER_CERTIFICATE: &str = include_str!("../certs/server.crt");
const SERVER_PRIVATE_KEY: &str = include_str!("../certs/server.key");

fn main() {
    let mut app = App::new();

    shared::setup(&mut app);

    app.add_systems(Startup, |mut endpoints: ResMut<Endpoints>, channels: Channels| {
        // let endpoint = EndpointBuilder::new()
        //     .bind(SERVER_ADDRESS)
        //     .with_channel_registry(channels.clone_arc())
        //     .use_existing(Arc::new(EndpointConfig::default()))
        //     .server_from_config(ServerConfig::with_single_cert(
        //         vec![
        //             shared::certificate(SERVER_CERTIFICATE),
        //         ],
        //         private_key(SERVER_PRIVATE_KEY),
        //     ).unwrap().into());

        // endpoints.waiting.insert(endpoint);
    });

    app.add_systems(Update, |mut endpoints: ResMut<Endpoints>, mut handler: ResMut<EndpointHandler>| {
        // while let Some(endpoint) = endpoints.waiting.poll() {
        //     let endpoint = endpoint.unwrap();
        //     handler.insert(endpoint.clone().downgrade());
        //     endpoints.endpoints.push(endpoint);
        // }
    });

    app.run();
}