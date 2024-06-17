pub use std::sync::Arc;
pub use std::net::{UdpSocket, SocketAddr};
pub use bevy::prelude::*;
pub use bevy_stardust::prelude::*;
pub use bevy_stardust_quic::*;

use std::net::{IpAddr, Ipv4Addr};
use bevy::app::ScheduleRunnerPlugin;
use bevy::log::LogPlugin;

pub const SERVER_ADDRESS: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 12345);
pub const RANDOM_ADDRESS: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);

pub fn setup_app() -> App {
    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins.build().disable::<LogPlugin>(),
        LogPlugin {
            filter: "".to_string(),
            level: bevy::log::Level::TRACE,
            update_subscriber: None,
        },
        ScheduleRunnerPlugin::run_loop(std::time::Duration::from_millis(200)),
        StardustPlugin,
        QuicPlugin,
    ));

    return app;
}

// This is not meant to be used.
#[allow(unused)]
fn main() { unimplemented!() }