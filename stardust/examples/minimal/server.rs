use std::net::{Ipv4Addr, IpAddr};
use bevy::prelude::*;
use bevy_stardust::{server::{prelude::*, transport::udp::ServerUdpTransportPlugin}, shared::channels::outgoing::SendTarget};
use crate::{RandomDataChannel, apply_shared_data, gen_random_string};

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
    clients: Query<(), With<Client>>,
    mut writer: ChannelWriter<RandomDataChannel>,
) {
    if clients.is_empty() { return; }
    
    let string = gen_random_string();

    info!("Sent data to clients: {string}");
    let _ = writer.send(SendTarget::Broadcast, string.into_bytes());
}

fn receive_random_data_system_server(
    reader: ChannelReader<RandomDataChannel>,
) {
    let iter = reader.read_all();
    for (client, messages) in iter {
        for message in messages.0.iter() {
            let string = String::from_utf8_lossy(message.read());
            info!("Received message from {:?}: {}", client, &string);
        }
    }
}