mod shared;

use bevy::prelude::*;
use bevy_stardust_quic::*;

const PRIVATE_KEY: &str = include_str!("../private_key.key");

fn main() {
    let mut app = shared::app();

    app.run();
}