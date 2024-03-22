#![allow(unused)]

use std::{net::{Ipv4Addr, SocketAddr, SocketAddrV4}, time::Duration};
use bevy_app::{prelude::*, ScheduleRunnerPlugin};
use bevy_ecs::prelude::*;
use bevy_log::LogPlugin;
use bevy_reflect::prelude::*;
use bevy_stardust::prelude::*;
use bevy_stardust_udp::*;

pub const LISTENER_ADDRESS: &str = "127.0.0.1:12345";
pub const UNSPECIFIED_SOCKET_ADDR: SocketAddr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0));

#[derive(Reflect)]
pub struct MyChannel;

pub fn setup_app() -> App {
    let mut app = App::new();

    app.add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_millis(100)));
    app.add_plugins(LogPlugin {
        filter: "".to_string(),
        level: tracing::Level::TRACE,
        update_subscriber: None,
    });

    app.add_plugins(StardustPlugin);

    app.add_channel::<MyChannel>(ChannelConfiguration {
        reliable: ReliabilityGuarantee::Reliable,
        ordered: OrderingGuarantee::Ordered,
        fragmented: false,
        priority: 0xFF,
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

fn send_and_recv_system(
    registry: ChannelRegistry,
    mut peers: Query<(Entity, &NetworkMessages<Incoming>, &mut NetworkMessages<Outgoing>), With<NetworkPeer>>,
) {
    for (origin, incoming, mut outgoing) in peers.iter_mut() {
        // Get the ID for our channel
        let id = registry.channel_id(std::any::TypeId::of::<MyChannel>()).unwrap();

        // Read all messages
        for message in incoming.channel_queue(id) {
            let message = std::str::from_utf8(&message).unwrap();
            tracing::info!("Received a message from {origin:?}: {message}");
        }

        // Send a message to the peer
        outgoing.push(id, Bytes::from_static(b"Hello, world!"));
    }
}