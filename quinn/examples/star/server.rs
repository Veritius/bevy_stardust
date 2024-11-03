// mod shared;

// use std::sync::Arc;
// use bevy_app::prelude::*;
// use bevy_ecs::prelude::*;
// use bevy_stardust_quinn::*;
// use quinn_proto::{EndpointConfig, ServerConfig};
// use shared::*;

// // NOTE: It is very, very, very bad practice to compile-in private keys.
// // This is only done for the sake of simplicity. In reality, you should
// // get private keys and certificates from files.
// const SERVER_CERTIFICATE: &str = include_str!("../certs/server.crt");
// const SERVER_PRIVATE_KEY: &str = include_str!("../certs/server.key");

// fn main() {
//     let mut app = App::new();

//     shared::setup(&mut app);

//     app.add_systems(Startup, |mut commands: Commands| {
//         commands.spawn_empty().make_endpoint(MakeEndpoint {
//             socket: QuicSocket::new(SERVER_ADDRESS).unwrap(),
//             config: Arc::new(EndpointConfig::default()),
//             server: Some(Arc::new(ServerConfig::with_single_cert(
//                 vec![
//                     shared::certificate(SERVER_CERTIFICATE),
//                 ],
//                 private_key(SERVER_PRIVATE_KEY),
//             ).unwrap())),
//         });
//     });

//     app.run();
// }

fn main() {}