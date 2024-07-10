use std::time::Duration;
use bevy::{app::ScheduleRunnerPlugin, prelude::*};
use bevy_stardust::prelude::*;
use bevy_stardust_quic::*;

const TICK_DELAY: Duration = Duration::from_millis(500);

const CERTIFICATE: &str = include_str!("certificate.crt");
const PRIVATE_KEY: &str = include_str!("private_key.key");

struct MyChannel;

fn main() {

}

fn client() -> App {
    let mut app = shared();

    return app;
}

fn server() -> App {
    let mut app = shared();

    return app;
}

fn shared() -> App {
    let mut app = App::new();

    app.add_plugins(ScheduleRunnerPlugin::run_loop(TICK_DELAY));
    app.add_plugins(bevy::app::MainSchedulePlugin);

    app.add_plugins(StardustPlugin);

    app.add_channel::<MyChannel>(ChannelConfiguration {
        consistency: ChannelConsistency::ReliableOrdered,
        priority: 0,
    });

    app.add_plugins(QuicPlugin);

    return app;
}