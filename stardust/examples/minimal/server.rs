use std::net::{Ipv4Addr, IpAddr};
use bevy::prelude::*;
use bevy_stardust::server::{prelude::*, transport::udp::ServerUdpTransportPlugin};

use crate::{apply_shared_data, RandomDataChannel};

pub(super) fn server() -> App {
    let mut app = App::new();
    apply_shared_data(&mut app);

    app.add_plugins(StardustServerPlugin);
    app.add_plugins(ServerUdpTransportPlugin {
        address: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        listen_port: 12345,
        active_port: 12346,
    });

    // Add our sending and receiving systems
    app.add_systems(ReadOctetStrings, receive_random_data_system_server);
    app.add_systems(WriteOctetStrings, send_random_data_system_server);

    // Configure the server
    app.insert_resource(NetworkClientCap(64));

    app
}

fn send_random_data_system_server(
    writer: ChannelWriter<RandomDataChannel>,
) {
    
}

fn receive_random_data_system_server(
    reader: ChannelReader<RandomDataChannel>,
) {
    
}