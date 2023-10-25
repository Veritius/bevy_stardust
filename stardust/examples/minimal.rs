use std::{net::{SocketAddr, IpAddr, Ipv4Addr}, time::Duration};

use bevy::{prelude::*, log::LogPlugin, app::SubApp};
use bevy_stardust::{setup::StardustPlugin, prelude::{UdpTransportPlugin, UdpConnectionManager}, transports::udp::startup_now};

const SERVER_PORTS: &[u16] = &[12340];
const CLIENT_PORTS: &[u16] = &[12349];

fn main() {
    // Create host app that runs both peers
    let mut host = App::new();
    host.add_plugins((MinimalPlugins, LogPlugin::default()));

    // Create server and client apps
    let mut server = App::new();
    let mut client = App::new();

    // Apply shared data
    for app in [&mut server, &mut client] {
        app.add_plugins((MinimalPlugins, StardustPlugin, UdpTransportPlugin));
    }

    // Put our plugins into standby immediately
    // You probably don't want to use this method for your game.
    // startup_now is intended for dedicated servers (and simpler examples)
    startup_now(&mut server.world, None, SERVER_PORTS);
    startup_now(&mut client.world, None, CLIENT_PORTS);

    // Instruct the server to listen for new connections on startup
    server.add_systems(Startup, |mut manager: UdpConnectionManager| {
        // TODO: Commented out because toggling listening is not done yet
        // manager.enable_listening();
    });

    // Instruct the client to join the server on startup
    client.add_systems(Startup, |mut manager: UdpConnectionManager| {
        manager.connect_to_remote(
            SocketAddr::new(
                IpAddr::V4(Ipv4Addr::UNSPECIFIED),
                *SERVER_PORTS.iter().nth(0).unwrap()
            ), Some(Duration::from_secs(5)));
    });

    // Add subapps to host
    host.insert_sub_app("server", SubApp::new(server, |_, _| {}));
    host.insert_sub_app("client", SubApp::new(client, |_, _| {}));

    // Run the host indefinitely
    loop {
        host.update();
    }
}