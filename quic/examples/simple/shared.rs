#![allow(unused)]

use std::any::TypeId;
pub use std::sync::Arc;
pub use std::net::{UdpSocket, SocketAddr};
pub use bevy::prelude::*;
use bevy::utils::HashMap;
pub use bevy_stardust::prelude::*;
pub use bevy_stardust_quic::*;

use std::net::{IpAddr, Ipv4Addr};
use bevy::app::ScheduleRunnerPlugin;
use bevy::log::LogPlugin;

pub const SERVER_ADDRESS: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 12345);
pub const RANDOM_ADDRESS: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);

pub fn setup_app() -> App {
    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins.build().disable::<LogPlugin>(),
        LogPlugin {
            filter: "".to_string(),
            level: bevy::log::Level::DEBUG,
            custom_layer: |_| None,
        },
        ScheduleRunnerPlugin::run_loop(std::time::Duration::from_millis(200)),
        StardustPlugin,
        QuicPlugin,
    ));

    app.add_channel::<UnreliableUnorderedChannel>(ChannelConfiguration {
        consistency: ChannelConsistency::UnreliableUnordered,
        priority: 0,
    });

    app.add_channel::<UnreliableSequencedChannel>(ChannelConfiguration {
        consistency: ChannelConsistency::UnreliableSequenced,
        priority: 0,
    });


    app.add_channel::<ReliableUnorderedChannel>(ChannelConfiguration {
        consistency: ChannelConsistency::UnreliableUnordered,
        priority: 0,
    });


    app.add_channel::<ReliableOrderedChannel>(ChannelConfiguration {
        consistency: ChannelConsistency::ReliableOrdered,
        priority: 0,
    });

    app.add_systems(Update, send_recv_message_system);

    return app;
}

struct UnreliableUnorderedChannel;
struct UnreliableSequencedChannel;
struct ReliableUnorderedChannel;
struct ReliableOrderedChannel;

fn send_recv_message_system(
    channels: Channels,
    mut increments: Local<HashMap<ChannelId, u64>>,
    mut peers: Query<(
        Entity,
        &PeerMessages<Incoming>,
        &mut PeerMessages<Outgoing>,
    )>,
) {
    const SEND_CHANCE: f32 = 0.8;

    let mut rng = fastrand::Rng::default();

    for (entity, incoming, mut outgoing) in peers.iter_mut() {
        // Read out all messages
        let iter = incoming.iter().flat_map(|(c, m)| m.map(move |v| (c, v)));
        for (channel, message) in iter {
            let str = std::str::from_utf8(&message[..]).unwrap();
            info!("Received message from {entity} on channel {channel:?}: {str}")
        }

        // Send messages on each channel
        for channel in (0..channels.count()).map(|v| ChannelId::from(v)) {
            let increment = increments.entry(channel).or_insert(0);

            for _ in (0..rng.u32(0..4)) {
                let message = format!("This is message {}", *increment);
                info!("Sending message to {entity} on channel {channel:?}: {message}");
                let payload = Message::from(Bytes::from(message));
                outgoing.push_one(ChannelMessage { channel, payload });
                *increment += 1;
            }
        }
    }
}

// This is not meant to be used.
#[allow(unused)]
fn main() { unimplemented!() }