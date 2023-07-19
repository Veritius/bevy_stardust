use bevy_stardust_client::plugin::StardustClientPlugin;
use bevy_stardust_shared::{bevy::{prelude::App, DefaultPlugins}, plugin::StardustSharedPlugin};
use bevy_stardust_shared_simpledemo::DemoSharedPlugin;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);

    // Note: Order matters!
    // When adding any plugins that change the Protocol, they must always be added after StardustSharedPlugin and StardustClientPlugin

    app.add_plugins(StardustSharedPlugin);
    app.add_plugins(StardustClientPlugin {});

    app.add_plugins(DemoSharedPlugin);
}