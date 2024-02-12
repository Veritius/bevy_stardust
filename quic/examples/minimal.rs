use std::thread;
use bevy::{ecs::schedule::ExecutorKind, prelude::*};
use bevy_stardust::{channels::config::{ChannelConfiguration, OrderingGuarantee, ReliabilityGuarantee}, plugin::StardustPlugin, prelude::ChannelSetupAppExt};
use bevy_stardust_quic::QuicTransportPlugin;

const SERVER_ADDRESS: &str = "127.0.0.1:12344";
const CLIENT_ADDRESS: &str = "127.0.0.1:12345";

#[derive(TypePath)]
struct MyMessage;

fn main() {
    // Client
    thread::spawn(|| {
        let mut app = setup_app();
        loop { app.update(); }
    });

    // Server
    thread::spawn(|| {
        let mut app = setup_app();
        loop { app.update(); }
    });

    // spin infinitely
    // todo: remove this
    loop {}
}

fn setup_app() -> App {
    let mut app = App::new();
    app.edit_schedule(Main, |f| {
        f.set_executor_kind(ExecutorKind::SingleThreaded) ;
    });

    app.add_plugins(MinimalPlugins);
    app.add_plugins(StardustPlugin);

    app.add_channel::<MyMessage>(ChannelConfiguration {
        reliable: ReliabilityGuarantee::Reliable,
        ordered: OrderingGuarantee::Ordered,
        fragmented: false,
        string_size: 0..=128,
    });

    app.add_plugins(QuicTransportPlugin {
        allow_self_signed: true,
        reliable_streams: 8,
        timeout_delay: 30,
    });

    app
}