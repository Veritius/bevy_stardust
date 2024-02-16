#![allow(unused)] // rustc doesn't detect usage in other examples

use std::io::Cursor;
use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_log::LogPlugin;
use bevy_stardust::prelude::*;
use bevy_stardust_quic::*;
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
        fragmented: false,
        string_size: 0..=128,
    });

    app.add_plugins(QuicTransportPlugin {
        authentication: TlsAuthentication::AlwaysVerify,
        reliable_streams: 8,
        transport_config_override: None,
    });

    app
}

pub fn root_cert_store() -> RootCertStore {
    RootCertStore::empty()
}