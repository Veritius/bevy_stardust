mod shared;

use bevy::prelude::*;

// NOTE: It is very, very, very bad practice to compile-in private keys.
// This is only done for the sake of simplicity. In reality, you should
// get private keys and certificates from files.
const PRIVATE_KEY: &str = include_str!("../../certs/private_key.key");

fn main() {
    let mut app = App::new();

    shared::setup(&mut app);

    app.run();
}