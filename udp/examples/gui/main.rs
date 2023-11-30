//! A simple client-server chat application.

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_stardust_udp::prelude::*;

fn main() {
    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins,
        EguiPlugin,
        UdpTransportPlugin,
    ));

    todo!();
}