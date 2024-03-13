use std::{net::{SocketAddr, SocketAddrV4, Ipv4Addr}, time::Duration};
use bevy_app::{prelude::*, AppLabel, ScheduleRunnerPlugin, SubApp};
use bevy_log::LogPlugin;
use bevy_stardust::plugin::StardustPlugin;
use bevy_stardust_udp::*;

const LISTENER_ADDRESS: &str = "127.0.0.1:12345";
const UNSPECIFIED_SOCKET_ADDR: SocketAddr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0));

fn main() {
    let mut initiator = setup_app();

    initiator.add_systems(Startup, |mut manager: UdpManager| {
        manager.open_endpoint_and_connect(UNSPECIFIED_SOCKET_ADDR, LISTENER_ADDRESS).unwrap();
    });

    let mut listener = setup_app();

    listener.add_systems(Startup, |mut manager: UdpManager| {
        manager.open_endpoint(LISTENER_ADDRESS, true).unwrap();
    });

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, AppLabel)]
    enum AppLabel {
        Initiator,
        Listener,
    }

    let mut manager = App::new();

    manager.add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_secs(1)));
    manager.add_plugins(LogPlugin::default());

    manager.insert_sub_app(AppLabel::Initiator, SubApp::new(initiator, |_,_| {}));
    manager.insert_sub_app(AppLabel::Listener, SubApp::new(listener, |_,_| {}));

    manager.run();
}

fn setup_app() -> App {
    let mut app = App::new();

    app.add_plugins(StardustPlugin);
    app.add_plugins(UdpTransportPlugin::balanced(123456789));

    return app;
}