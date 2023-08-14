use bevy::prelude::*;
use bevy_stardust::client::{prelude::*, transport::udp::{ClientUdpTransportPlugin, UdpConnectionManager}};
use rand::RngCore;

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
    mut writer: ChannelWriter<RandomDataChannel>,
) {
    let mut rng = rand::thread_rng();
    let mut octets = Vec::with_capacity(256);

    rng.fill_bytes(octets.as_mut());
    let _ = writer.send(octets);
}

fn receive_random_data_system_client(
    reader: ChannelReader<RandomDataChannel>,
) {
    let payloads = reader.read_from_server();
    let Ok(Some(payloads)) = payloads else { return };

    for payload in payloads.0.iter() {
        let string = String::from_utf8_lossy(payload.read());
        info!("Received data from server: {}", &string);
    }
}