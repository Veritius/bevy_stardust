mod setup;

use bevy::prelude::*;
use bevy_stardust::prelude::*;
use bevy_stardust_replicate::prelude::*;

fn main() {
    let side = match detect_side() {
        Some(v) => v,
        None => {
            println!("No side picked. Use --server or --client as an argument.");
            return;
        },
    };

    let mut app = App::new();
    app.add_plugins((DefaultPlugins, StardustPlugin, CoreReplicationPlugin));

    app.add_systems(Startup, setup::spawn_camera);

    app.run();
}

fn detect_side() -> Option<Side> {
    // Detect what side we use from stdin
    let mut side = None;
    let mut args = std::env::args();
    args.next(); // Ignore the first item.
    for arg in args {
        match arg.as_str() {
            "--server" => {
                println!("Running as a server");
                side = Some(Side::Server);
                break;
            }
            "--client" => {
                println!("Running as a client");
                side = Some(Side::Client);
                break;
            }
            _ => {
                println!("Didn't recognise argument: {arg:?}");
            }
        }
    }

    return side;
}