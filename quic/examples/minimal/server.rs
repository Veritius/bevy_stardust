mod shared;
use shared::*;

use bevy::prelude::*;
use bevy_stardust::prelude::*;
use bevy_stardust_quic::*;

fn main() {
    let mut server = setup_app();

    // server.add_systems(Startup, |mut manager: QuicConnectionManager| {
    //     manager.open_server_endpoint(
    //         SERVER_ADDRESS,
    //         certs.0.clone(),
    //         vec![pair.0.clone()],
    //         pair.1.clone()
    //     ).unwrap();
    // });
}