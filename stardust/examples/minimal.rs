use bevy::prelude::*;

use bevy_stardust::client::prelude::*;
use bevy_stardust::client::transport::udp::ClientUdpTransportPlugin;

use bevy_stardust::server::prelude::*;
use bevy_stardust::server::transport::udp::ServerUdpTransportPlugin;

fn main() {
    let mut server = server();
    let mut client = client();

    loop {
        server.update();
        client.update();
    }
}

fn client() -> App {
    let mut app = App::new();
    apply_shared_data(&mut app);

    app.add_plugins(StardustClientPlugin);
    app.add_plugins(ClientUdpTransportPlugin);

    app
}

fn server() -> App {
    let mut app = App::new();
    apply_shared_data(&mut app);

    app.add_plugins(StardustServerPlugin);
    app.add_plugins(ServerUdpTransportPlugin {
        listen_port: 12345,
        active_port: 12345,
    });

    // Configure the server
    app.insert_resource(NetworkClientCap(64));

    app
}

/// Applies information that is identical on both the client and server to the App.
fn apply_shared_data(app: &mut App) {
    // Add plugins
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StardustSharedPlugin);

    // Add channel
    app.register_channel::<RandomDataChannel>(ChannelConfig {
        direction: ChannelDirection::Bidirectional,
    }, ());
}

/// Random data, bidirectionally.
#[derive(Debug, Reflect)]
struct RandomDataChannel;