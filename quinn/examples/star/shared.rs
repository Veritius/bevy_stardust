use bevy_ecs::prelude::*;
use bevy_app::{prelude::*, ScheduleRunnerPlugin};
use bevy_log::LogPlugin;
use bevy_stardust::prelude::*;
use bevy_stardust_quinn::{CertificateDer, QuinnPlugin};
use rustls_pemfile::Item;

// NOTE: It is very, very, very bad practice to compile-in certificates.
// This is only done for the sake of simplicity. In reality, you should
// get private keys and certificates from files.
pub const CERTIFICATE: &str = include_str!("../certs/certificate.crt");

pub const SERVER_ADDRESS: &str = "127.0.0.1:12345";
pub const WILDCARD_ADDRESS: &str = "127.0.0.1:0";

pub fn certificate() -> CertificateDer<'static> {
    match rustls_pemfile::read_one_from_slice(CERTIFICATE.as_bytes()) {
        Ok(Some((Item::X509Certificate(cert), _))) => return cert,
        _ => panic!(),
    }
}

pub fn setup(app: &mut App) {
    app.add_plugins((
        LogPlugin {
            level: bevy_log::Level::TRACE,
            ..Default::default()
        },
        ScheduleRunnerPlugin::default(),
        StardustPlugin,
        QuinnPlugin,
    ));

    app.add_channel::<MovementEvent>(ChannelConfiguration {
        consistency: MessageConsistency::UnreliableSequenced,
        priority: 32,
    });

    app.add_event::<MovementEvent>();
}

#[derive(Debug, Event)]
pub struct MovementEvent {
    direction: [f32; 2],
}