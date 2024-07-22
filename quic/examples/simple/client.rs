mod shared;

use std::net::{IpAddr, Ipv4Addr};
use bevy::prelude::*;
use bevy_stardust_quic::*;

fn main() {
    let mut app = shared::app();

    app.run();
}