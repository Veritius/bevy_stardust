use bevy::prelude::*;
use bevy_stardust::{prelude::client::*, scheduling::*, transports::udp::prelude::*, client::{connection::RemoteConnectionStatus, send::ChannelWriter}, plugin::StardustPlugin};
use crate::{apply_shared_data, gen_random_string, RandomDataChannel};

pub(super) fn client() -> App {
    let mut app = App::new();

    app.add_plugins(StardustPlugin::DedicatedClient);
    app.add_plugins(ClientUdpTransportPlugin);

    apply_shared_data(&mut app);

    // Add our sending and receiving systems
    app.add_systems(ReadOctetStrings, receive_random_data_system_client);
    app.add_systems(Update, send_random_data_system_client);

    app.add_systems(Startup, |mut manager: UdpConnectionManager| {
        use std::net::*;
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        manager.join(SocketAddr::new(ip, 12345));
    });

    app
}

fn send_random_data_system_client(
    conn: Res<State<RemoteConnectionStatus>>,
    mut writer: ChannelWriter<RandomDataChannel>,
) {
    if !conn.connected() { return; }

    let string = gen_random_string();

    info!("Sent data to server: {string}");
    let _ = writer.send(string.into_bytes());
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