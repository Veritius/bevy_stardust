mod shared;

use std::sync::Arc;
use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_stardust_quinn::*;
use quinn_proto::{EndpointConfig, ServerConfig};
use rustls_pemfile::Item;
use shared::*;

// NOTE: It is very, very, very bad practice to compile-in private keys.
// This is only done for the sake of simplicity. In reality, you should
// get private keys and certificates from files.
const PRIVATE_KEY: &str = include_str!("../certs/private_key.key");

fn private_key() -> PrivateKeyDer<'static> {
    match rustls_pemfile::read_one_from_slice(PRIVATE_KEY.as_bytes()) {
        Ok(Some((Item::Pkcs1Key(key), _))) => return key.into(),
        Ok(Some((Item::Pkcs8Key(key), _))) => return key.into(),
        Ok(Some((Item::Sec1Key(key), _))) => return key.into(),
        _ => panic!(),
    }
}

fn main() {
    let mut app = App::new();

    shared::setup(&mut app);

    app.add_systems(Startup, |mut commands: Commands| {
        commands.spawn_empty().make_endpoint(MakeEndpoint {
            socket: QuicSocket::new(SERVER_ADDRESS).unwrap(),
            config: Arc::new(EndpointConfig::default()),
            server: Some(Arc::new(ServerConfig::with_single_cert(
                vec![shared::certificate()],
                private_key(),
            ).unwrap())),
        });
    });

    app.run();
}