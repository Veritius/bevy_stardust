#![allow(unused)] // rustc doesn't detect usage in other examples

use std::io::Cursor;
use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_log::LogPlugin;
use bevy_stardust::prelude::*;
use bevy_stardust_quic::*;
use fastrand::Rng;
use rustls::{Certificate, PrivateKey, RootCertStore};

pub const SERVER_ALT_NAME: &str = "www.icann.org";
pub const SERVER_ADDRESS: &str = "localhost:12345";
pub const CLIENT_ADDRESS: &str = "localhost:0";

pub struct MyMessage;

pub fn setup_app() -> App {
    let mut app = App::new();
    app.edit_schedule(Main, |f| {
        // We don't need parallelism here.
        f.set_executor_kind(bevy_ecs::schedule::ExecutorKind::SingleThreaded) ;
    });

    app.add_plugins(bevy_app::ScheduleRunnerPlugin {
        run_mode: bevy_app::RunMode::Loop { wait: None }
    });

    app.add_plugins(LogPlugin {
        level: tracing::Level::INFO,
        filter: "".to_string(),
    });

    app.add_plugins(StardustPlugin);

    app.add_channel::<MyMessage>(ChannelConfiguration {
        reliable: ReliabilityGuarantee::Reliable,
        ordered: OrderingGuarantee::Ordered,
        priority: 0,
        fragmented: false,
        string_size: 0..=128,
    });

    app.add_plugins(QuicTransportPlugin {
        authentication: TlsAuthentication::AlwaysVerify,
        reliable_streams: 8,
        transport_config_override: None,
    });

    app.add_systems(Update, exchange_messages_system);

    app
}

fn exchange_messages_system(
    other: Query<Entity, With<NetworkPeer>>,
    mut rng: Local<Rng>,
    mut reader: NetworkReader<MyMessage>,
    mut writer: NetworkWriter<MyMessage>,
) {
    // A set of random (UTF-8) strings we can choose from
    static GREEK_ALPHABET: &[str] = [
        "Alpha", "Beta", "Gamma", "Delta", "Epsilon", "Zeta", "Eta", "Theta",
        "Iota", "Kappa", "Lambda", "Mu", "Nu", "Xi", "Omicron", "Pi", "Rho",
        "Sigma", "Tau", "Upsilon", "Phi", "Chi", "Psi", "Omega",
    ];

    // Read messages and print them to the console
    for (origin, message) in reader.iter() {
        let string = std::str::from_utf8(&message).unwrap();
        tracing::info!("Received a message from {origin}: {string}");
    }

    // Write some random data to send to our friend
    if let Some(id) = other.get_single() {
        let mut scratch = String::new();
        for i in 0..8 {
            scratch.push_str(fastrand::choice(GREEK_ALPHABET).unwrap());
        }

        writer.send(id, Bytes::from(scratch));
    }
}

pub fn root_cert_store() -> RootCertStore {
    RootCertStore::empty()
}