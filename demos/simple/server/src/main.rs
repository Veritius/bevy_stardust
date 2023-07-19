use bevy_stardust_server::{plugin::StardustServerPlugin, config::ServerConfig};
use bevy_stardust_shared::{bevy::{prelude::App, MinimalPlugins}, plugin::StardustSharedPlugin};
use bevy_stardust_shared_simpledemo::DemoSharedPlugin;

fn main() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Note: Order matters!
    // When adding any plugins that change the Protocol, they must always be added after StardustSharedPlugin and StardustServerPlugin

    app.add_plugins(StardustSharedPlugin);
    app.add_plugins(StardustServerPlugin {
        config: ServerConfig {
            max_players: 64,
        },
        bind_port: 26020,
    });

    app.add_plugins(DemoSharedPlugin);
}