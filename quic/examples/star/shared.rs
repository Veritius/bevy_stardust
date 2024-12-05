#![allow(dead_code)]

use std::{net::{IpAddr, Ipv4Addr, SocketAddr}, time::Duration};
use bevy_ecs::prelude::*;
use bevy_app::{prelude::*, ScheduleRunnerPlugin};
use bevy_log::{info, LogPlugin};
use bevy_stardust::prelude::*;
use bevy_stardust_quic::*;
use rustls_pemfile::Item;

// NOTE: It is very, very, very bad practice to compile-in certificates.
// This is only done for the sake of simplicity. In reality, you should
// get private keys and certificates from files.
pub const CA_CERTIFICATE: &str = include_str!("../certs/ca.crt");

pub const SERVER_ADDRESS: SocketAddr = SocketAddr::new(
    IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 12345);

pub const WILDCARD_ADDRESS: SocketAddr = SocketAddr::new(
    IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
    

pub fn certificate(str: &'static str) -> CertificateDer<'static> {
    match rustls_pemfile::read_one_from_slice(str.as_bytes()) {
        Ok(Some((Item::X509Certificate(cert), _))) => return cert,
        _ => panic!(),
    }
}

pub fn private_key(str: &'static str) -> PrivateKeyDer<'static> {
    match rustls_pemfile::read_one_from_slice(str.as_bytes()) {
        Ok(Some((Item::Pkcs1Key(key), _))) => return key.into(),
        Ok(Some((Item::Pkcs8Key(key), _))) => return key.into(),
        Ok(Some((Item::Sec1Key(key), _))) => return key.into(),
        _ => panic!(),
    }
}

pub fn setup(app: &mut App) {
    app.add_plugins((
        LogPlugin {
            level: bevy_log::Level::TRACE,
            ..Default::default()
        },
        ScheduleRunnerPlugin::run_loop(
            Duration::from_millis(100),
        ),
        StardustPlugin,
        QuicPlugin,
    ));

    app.add_channel::<UnreliableUnordered>(ChannelConfiguration {
        consistency: MessageConsistency::UnreliableUnordered,
        priority: 32,
    });

    app.add_channel::<UnreliableSequenced>(ChannelConfiguration {
        consistency: MessageConsistency::UnreliableSequenced,
        priority: 32,
    });

    app.add_channel::<ReliableUnordered>(ChannelConfiguration {
        consistency: MessageConsistency::ReliableUnordered,
        priority: 32,
    });

    app.add_channel::<ReliableOrdered>(ChannelConfiguration {
        consistency: MessageConsistency::ReliableOrdered,
        priority: 32,
    });

    app.add_systems(Update, send_and_receive_system::<UnreliableUnordered>);
    app.add_systems(Update, send_and_receive_system::<UnreliableSequenced>);
    app.add_systems(Update, send_and_receive_system::<ReliableUnordered>);
    app.add_systems(Update, send_and_receive_system::<ReliableOrdered>);

    app.insert_resource(RuntimeBuilder::new()
        .threads(1)
        .build());
}

enum UnreliableUnordered {}
enum UnreliableSequenced {}
enum ReliableUnordered {}
enum ReliableOrdered {}

fn send_and_receive_system<C: Channel>(
    channel: ChannelData<C>,
    mut increment: Local<u32>,
    mut connections: Query<(
        Entity,
        &PeerMessages<Incoming>,
        &mut PeerMessages<Outgoing>,
    ), (
        With<Connection>,
        With<Peer>,
    )>,
) {
    for (
        entity,
        incoming,
        mut outgoing,
    ) in connections.iter_mut() {
        for (channel, messages) in incoming.iter() {
            for message in messages {
                let message = message.as_str().unwrap();
                info!("Received message from {entity} on channel {channel:?}: {message}");
            }
        }

        let value = *increment; *increment += 1;

        let channel = channel.id();
        let string = format!("{value:X}");
        info!("Sending message to {entity} on channel {channel:?}: {string}");
    
        outgoing.push_one(ChannelMessage {
            channel,
            message: Message::from_bytes(string.into()),
        });
    }
}