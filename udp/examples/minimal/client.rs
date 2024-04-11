mod shared;
use shared::*;

use bevy::prelude::*;
use bevy_stardust_udp::prelude::*;

fn main() {
    let mut app = setup_app();

    app.add_systems(Startup, |mut manager: UdpManager| {
        manager.open_endpoint_and_connect(UNSPECIFIED_SOCKET_ADDR, LISTENER_ADDRESS).unwrap();
    });

    app.run();
}