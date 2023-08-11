use std::net::{IpAddr, Ipv4Addr};

use bevy::app::SubApp;
use bevy::prelude::*;

use bevy_stardust::client::prelude::*;
use bevy_stardust::client::transport::udp::{ClientUdpTransportPlugin, UdpConnectionManager};

use bevy_stardust::server::prelude::*;
use bevy_stardust::server::transport::udp::ServerUdpTransportPlugin;

fn main() {
    let mut owner = App::new();
    owner.add_plugins(DefaultPlugins);

    owner.insert_sub_app("server", SubApp::new(server(), |_,_| {}));
    owner.insert_sub_app("client", SubApp::new(client(), |_,_| {}));

    loop { owner.update(); }
}

fn client() -> App {
    let mut app = App::new();
    apply_shared_data(&mut app);

    app.add_plugins(StardustClientPlugin);
    app.add_plugins(ClientUdpTransportPlugin);

    app.add_systems(Startup, |mut manager: UdpConnectionManager| {
        use std::net::*;
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        manager.join(SocketAddr::new(ip, 12345));
    });

    app
}

fn server() -> App {
    let mut app = App::new();
    apply_shared_data(&mut app);

    app.add_plugins(StardustServerPlugin);
    app.add_plugins(ServerUdpTransportPlugin {
        address: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
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