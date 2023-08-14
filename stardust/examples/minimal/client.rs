use bevy::prelude::*;
use bevy_stardust::client::{prelude::*, transport::udp::{ClientUdpTransportPlugin, UdpConnectionManager}};

use crate::{apply_shared_data, RandomDataChannel};

pub(super) fn client() -> App {
    let mut app = App::new();
    apply_shared_data(&mut app);

    app.add_plugins(StardustClientPlugin);
    app.add_plugins(ClientUdpTransportPlugin);

    // Add our sending and receiving systems
    app.add_systems(ReadOctetStrings, receive_random_data_system_client);
    app.add_systems(WriteOctetStrings, send_random_data_system_client);

    app.add_systems(Startup, |mut manager: UdpConnectionManager| {
        use std::net::*;
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        manager.join(SocketAddr::new(ip, 12345));
    });

    app
}

fn send_random_data_system_client(
    writer: ChannelWriter<RandomDataChannel>,
) {

}

fn receive_random_data_system_client(
    reader: ChannelReader<RandomDataChannel>,
) {

}