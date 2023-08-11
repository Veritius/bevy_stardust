use std::thread;

use bevy::prelude::*;

use bevy_stardust::client::prelude::*;
use bevy_stardust::client::transport::udp::ClientUdpTransportPlugin;

use bevy_stardust::server::prelude::*;
use bevy_stardust::server::transport::udp::ServerUdpTransportPlugin;

fn main() {
    start_server();
    start_client();
}

/// Creates a new App with client logic, running it on a separate thread.
fn start_client() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StardustSharedPlugin);
    app.add_plugins(StardustClientPlugin);
    app.add_plugins(ClientUdpTransportPlugin);

    register_channels(&mut app);

    thread::spawn(move || app.run());
}

/// Creates a new App with server logic, running it on a separate thread.
fn start_server() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StardustSharedPlugin);
    app.add_plugins(StardustServerPlugin);
    app.add_plugins(ServerUdpTransportPlugin {
        listen_port: 12345,
        active_port: 12345,
    });
    
    register_channels(&mut app);

    thread::spawn(move || app.run());
}

/// Registers all channels identically on both the client and server.
fn register_channels(app: &mut App) {
    app.register_channel::<RandomDataChannel>(ChannelConfig {
        direction: ChannelDirection::Bidirectional,
    }, ());
}

/// Random data, bidirectionally.
#[derive(Debug, Reflect)]
struct RandomDataChannel;