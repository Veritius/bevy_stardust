#![allow(unused)] // rustc doesn't detect usage in other examples

use std::io::Cursor;
use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_log::LogPlugin;
use bevy_stardust::prelude::*;
use bevy_stardust_quic::*;
use rustls::{Certificate, PrivateKey, RootCertStore};

pub const SERVER_ALT_NAME: &str = "www.icann.org";
pub const SERVER_ADDRESS: &str = "127.0.0.1:12344";
pub const CLIENT_ADDRESS: &str = "127.0.0.1:12345";

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
    app.add_plugins(LogPlugin::default());
    app.add_plugins(StardustPlugin);

    app.add_channel::<MyMessage>(ChannelConfiguration {
        reliable: ReliabilityGuarantee::Reliable,
        ordered: OrderingGuarantee::Ordered,
        fragmented: false,
        string_size: 0..=128,
    });

    app.add_plugins(QuicTransportPlugin {
        authentication: TlsAuthentication::Secure,
        reliable_streams: 8,
        timeout_delay: 30,
    });

    app
}

// All cryptographic information here is fully public for the sake of demonstration.
// Under no circumstances should you ever, ever, EVER use this in a real program.
// If you want to set up a real system, you should use 
static ROOT_CA: &str = include_str!("root-ca.crt");
static CERTIFICATE: &str = include_str!("server.crt");
static PRIVATE_KEY: &str = include_str!("server.key");

pub fn root_cert_store() -> RootCertStore {
    let mut store = RootCertStore::empty();
    let mut read = Cursor::new(ROOT_CA);
    let mut certs = rustls_pemfile::certs(&mut read).unwrap();
    let cert = Certificate(certs.remove(0));
    store.add(&cert).unwrap();
    store
}

pub fn certificate() -> Certificate {
    let mut read = Cursor::new(CERTIFICATE);
    let mut certs = rustls_pemfile::certs(&mut read).unwrap();
    Certificate(certs.remove(0))
}

pub fn private_key() -> PrivateKey {
    let mut read = Cursor::new(PRIVATE_KEY);
    let mut keys = rustls_pemfile::pkcs8_private_keys(&mut read).unwrap();
    PrivateKey(keys.remove(0))
}