use std::time::Duration;
use bevy::{app::ScheduleRunnerPlugin, log::LogPlugin, prelude::*};
use bevy_stardust::prelude::*;
use bevy_stardust_quic::*;

const TICK_DELAY: Duration = Duration::from_millis(500);

pub const CERTIFICATE: &str = include_str!("../certificate.crt");

pub enum MyChannel {}

pub fn app() -> App {
    let mut app = App::new();

    app.add_plugins(LogPlugin::default());

    app.add_plugins(ScheduleRunnerPlugin::run_loop(TICK_DELAY));

    app.add_plugins(StardustPlugin);

    app.add_channel::<MyChannel>(ChannelConfiguration {
        consistency: ChannelConsistency::ReliableOrdered,
        priority: 0,
    });

    app.add_plugins(QuicPlugin);

    return app;
}

pub fn app_protos() -> AppProtos {
    const APP_PROTOS: AppProtos = AppProtos::from_static_slice(APP_PROTOS_INNER);

    const APP_PROTOS_INNER: &'static [AppProto] = &[
        AppProto::from_static_str("simple_example"),
    ];

    APP_PROTOS
}

pub fn trust_anchors() -> TrustAnchors {
    TrustAnchors::from_iter(Some(Certificate::from_pem(CERTIFICATE).unwrap()))
}