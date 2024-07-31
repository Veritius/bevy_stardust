mod shared;

use bevy::prelude::*;

const PRIVATE_KEY: &str = include_str!("../certs/private_key.key");

fn main() {
    let mut app = App::new();

    shared::setup(&mut app);

    app.run();
}