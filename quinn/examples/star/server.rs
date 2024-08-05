mod shared;

use std::sync::Arc;

use bevy::prelude::*;
use bevy_stardust_quinn::{Manager, PrivateKeyDer};
use quinn_proto::{EndpointConfig, ServerConfig};
use rustls_pemfile::Item;

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

    app.add_systems(Startup, |mut manager: Manager| {
        manager.open_endpoint(
            Arc::new(EndpointConfig::default()),
            Some(Arc::new(ServerConfig::with_single_cert(
                vec![shared::certificate()],
                private_key(),
            ).unwrap())),
            shared::SERVER_ADDRESS,
        ).unwrap();
    });

    app.run();
}