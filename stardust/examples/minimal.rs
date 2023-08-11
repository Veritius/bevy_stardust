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

fn start_client() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StardustSharedPlugin);
    app.add_plugins(StardustClientPlugin);
    app.add_plugins(ClientUdpTransportPlugin);

    thread::spawn(move || app.run());
}

fn start_server() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StardustSharedPlugin);
    app.add_plugins(StardustServerPlugin);
    app.add_plugins(ServerUdpTransportPlugin {
        listen_port: 12345,
        active_port: 12345,
    });

    thread::spawn(move || app.run());
}