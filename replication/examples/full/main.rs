mod setup;

use bevy::prelude::*;
use bevy_stardust::prelude::*;
use bevy_stardust_replicate::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins((DefaultPlugins, StardustPlugin, CoreReplicationPlugin));

    app.add_systems(Startup, setup::spawn_camera);

    app.run();
}