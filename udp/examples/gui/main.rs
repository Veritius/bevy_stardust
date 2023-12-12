//! A simple client-server chat application.

use bevy::prelude::*;
use bevy_stardust::prelude::*;
use bevy_stardust_udp::prelude::*;
use bevy_egui::EguiPlugin;

fn main() {
    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins,
        StardustPlugin,
        UdpTransportPlugin::default(),
        EguiPlugin,
    ));

    todo!();
}