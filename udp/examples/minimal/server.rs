mod shared;
use shared::*;

use bevy::prelude::*;
use bevy_stardust_udp::*;

fn main() {
    let mut app = setup_app();

    app.add_systems(Startup, |mut manager: UdpManager| {
        manager.open_endpoint(LISTENER_ADDRESS, true).unwrap();
    });

    app.run();
}