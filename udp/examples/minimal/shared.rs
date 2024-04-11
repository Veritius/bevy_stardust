#![allow(unused)]

use std::{net::{Ipv4Addr, SocketAddr, SocketAddrV4}, time::Duration};
use bevy::{prelude::*, app::ScheduleRunnerPlugin};
use bevy::prelude::*;
use bevy_log::LogPlugin;
use bevy_stardust::prelude::*;
use bevy_stardust_udp::prelude::*;

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

static GREEK_ALPHABET: &[&str] = &[
    "alpha", "beta", "gamma", "delta", "epsilon", "zeta", "eta", "theta", "iota", "kappa",
    "lambda", "mu", "nu", "xi", "omicron", "pi", "sigma", "tau", "upsilon", "phi", "chi", "omega"
];

fn send_and_recv_system(
    registry: Res<ChannelRegistry>,
    mut peers: Query<(Entity, &NetworkMessages<Incoming>, &mut NetworkMessages<Outgoing>), With<NetworkPeer>>,
) {
    for (peer, incoming, mut outgoing) in peers.iter_mut() {
        // Get the ID for our channel
        let id = registry.channel_id(std::any::TypeId::of::<MyChannel>()).unwrap();

        // Read all messages
        for message in incoming.channel_queue(id) {
            let message = std::str::from_utf8(&message).unwrap();
            tracing::info!("Received a message from {peer:?}: {message}");
        }

        // Compose a message of random Greek words
        let length = fastrand::usize(1..10);
        let mut picks = Vec::with_capacity(length);
        for _ in 0..length { picks.push(*(fastrand::choice(GREEK_ALPHABET.iter()).unwrap())); }
        let string = picks.join(" ");

        // Send it to the peer
        tracing::info!("Sent a message to {peer:?}: {string}");
        outgoing.push(id, Bytes::from(string));
    }
}