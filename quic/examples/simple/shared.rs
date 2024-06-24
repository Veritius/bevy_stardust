use std::any::TypeId;
pub use std::sync::Arc;
pub use std::net::{UdpSocket, SocketAddr};
pub use bevy::prelude::*;
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
            level: bevy::log::Level::TRACE,
            update_subscriber: None,
        },
        ScheduleRunnerPlugin::run_loop(std::time::Duration::from_millis(200)),
        StardustPlugin,
        QuicPlugin::default(),
    ));

    app.add_channel::<SimpleChannel>(ChannelConfiguration {
        reliable: ReliabilityGuarantee::Reliable,
        ordered: OrderingGuarantee::Ordered,
        fragmented: false,
        priority: 0,
    });

    app.add_systems(Update, send_recv_message_system);

    return app;
}

#[derive(TypePath)]
struct SimpleChannel;

fn send_recv_message_system(
    registry: Res<ChannelRegistry>,
    mut increment: Local<u64>,
    mut peers: Query<(
        Entity,
        &NetworkMessages<Incoming>,
        &mut NetworkMessages<Outgoing>,
    )>,
) {
    for (entity, incoming, mut outgoing) in peers.iter_mut() {
        // Read out all messages
        let iter = incoming.iter().flat_map(|(c, m)| m.iter().cloned().map(move |v| (c, v)));
        for (channel, message) in iter {
            let str = std::str::from_utf8(&message[..]).unwrap();
            info!("Received message from {entity:?} on channel {channel:?}: {str}")
        }

        // Send a message
        let cid = registry.channel_id(TypeId::of::<SimpleChannel>()).unwrap();
        let message = format!("This is message {}", *increment);
        info!("Sending message to {entity:?} on channel {cid:?}: {message}");
        outgoing.push(cid, Bytes::from(message));
        *increment += 1;
    }
}

// This is not meant to be used.
#[allow(unused)]
fn main() { unimplemented!() }