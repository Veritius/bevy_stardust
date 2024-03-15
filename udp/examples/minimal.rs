use std::{net::{SocketAddr, SocketAddrV4, Ipv4Addr}, time::Duration};
use bevy_ecs::prelude::*;
use bevy_app::{prelude::*, AppLabel, ScheduleRunnerPlugin, SubApp};
use bevy_log::LogPlugin;
use bevy_reflect::TypePath;
use bevy_stardust::prelude::*;
use bevy_stardust_udp::*;

const LISTENER_ADDRESS: &str = "127.0.0.1:12345";
const UNSPECIFIED_SOCKET_ADDR: SocketAddr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0));

fn main() {
    let mut initiator = setup_app();

    initiator.add_systems(Startup, |mut manager: UdpManager| {
        manager.open_endpoint_and_connect(UNSPECIFIED_SOCKET_ADDR, LISTENER_ADDRESS).unwrap();
    });

    let mut listener = setup_app();

    listener.add_systems(Startup, |mut manager: UdpManager| {
        manager.open_endpoint(LISTENER_ADDRESS, true).unwrap();
    });

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, AppLabel)]
    enum AppLabel {
        Initiator,
        Listener,
    }

    let mut manager = App::new();

    manager.add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_secs(1)));
    manager.add_plugins(LogPlugin {
        filter: "".to_string(),
        level: tracing::Level::TRACE,
        update_subscriber: None,
    });

    manager.insert_sub_app(AppLabel::Listener, SubApp::new(listener, |_,_| {}));
    manager.insert_sub_app(AppLabel::Initiator, SubApp::new(initiator, |_,_| {}));

    manager.run();
}

fn setup_app() -> App {
    let mut app = App::new();

    app.add_plugins(StardustPlugin);

    app.add_channel::<MyChannel>(ChannelConfiguration {
        reliable: ReliabilityGuarantee::Reliable,
        ordered: OrderingGuarantee::Ordered,
        fragmented: false,
        priority: 0xFF,
        string_size: 0..=100,
    });

    app.add_plugins(UdpTransportPlugin::balanced(ApplicationNetworkVersion {
        // These values are irrelevant since the minimal example never interfaces with an older version of itself.
        // If you're making a real app, read the NetworkVersionData documentation to understand the purpose of this.
        ident: 0x0,
        major: 0x0,
        minor: 0x0,
        banlist: &[],
    }));

    app.add_systems(Update, send_and_recv_system);

    return app;
}

#[derive(TypePath)]
struct MyChannel;

fn send_and_recv_system(
    peers: Query<Entity, With<NetworkPeer>>,
    mut writer: NetworkWriter<MyChannel>,
    reader: NetworkReader<MyChannel>,
) {
    // Read all messages
    for (origin, message) in reader.iter() {
        let message = std::str::from_utf8(&message).unwrap();
        tracing::info!("Received a message from {origin:?}: {message}");
    }

    // Send a message to all peers
    for peer in peers.iter() {
        writer.send(peer, Bytes::from_static(b"Hello, world!"));
    }
}