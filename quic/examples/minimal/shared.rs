use std::io::Cursor;
use bevy::{ecs::schedule::ExecutorKind, log::LogPlugin, prelude::*};
use bevy_stardust::prelude::*;
use bevy_stardust_quic::*;
use rustls::{Certificate, PrivateKey};

pub const SERVER_ALT_NAME: &str = "www.icann.org";
pub const SERVER_ADDRESS: &str = "127.0.0.1:12344";
pub const CLIENT_ADDRESS: &str = "127.0.0.1:12345";

#[derive(TypePath)]
pub struct MyMessage;

pub fn setup_app() -> App {
    let mut app = App::new();
    app.edit_schedule(Main, |f| {
        // We don't need parallelism here.
        f.set_executor_kind(ExecutorKind::SingleThreaded) ;
    });

    app.add_plugins((MinimalPlugins, LogPlugin::default()));
    app.add_plugins(StardustPlugin);

    app.add_channel::<MyMessage>(ChannelConfiguration {
        reliable: ReliabilityGuarantee::Reliable,
        ordered: OrderingGuarantee::Ordered,
        fragmented: false,
        string_size: 0..=128,
    });

    app.add_plugins(QuicTransportPlugin {
        reliable_streams: 8,
        timeout_delay: 30,
    });

    app
}

// This certificate is self signed and the key is public for the sake of demonstration.
// Under no circumstances should you ever, ever, EVER use this in a real program.
static CERTIFICATE: &str = include_str!("certificate.crt");
static PRIVATE_KEY: &str = include_str!("private_key.key");

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