mod client;
mod server;

use std::time::Duration;
use bevy::app::SubApp;
use bevy::prelude::*;
use bevy_stardust::shared::prelude::*;
use rand::Rng;
use rand::seq::SliceRandom;

/// How many times the loop runs per second.
const RUN_HZ: f64 = 2.0;

fn main() {
    let sleep_time = Duration::from_secs_f64(1.0 / RUN_HZ);

    let mut owner = App::new();
    owner.add_plugins(DefaultPlugins);
    owner.set_runner(move |mut app| loop {
        app.update();
        std::thread::sleep(sleep_time);
    });

    owner.insert_sub_app("server", SubApp::new(server::server(), |_,_| {}));
    owner.insert_sub_app("client", SubApp::new(client::client(), |_,_| {}));

    owner.run();
}

/// Applies information that is identical on both the client and server to the App.
fn apply_shared_data(app: &mut App) {
    // Add plugins
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StardustSharedPlugin);

    // Add channel
    app.register_channel::<RandomDataChannel>(ReliableChannel);
}

/// Random data, bidirectionally.
#[derive(Debug, Reflect)]
struct RandomDataChannel;

/// The greek alphabet, used for random string generation.
pub static GREEK_ALPHABET: &'static [&'static str] = &[
    "Alpha", "Beta", "Gamma",
    "Delta", "Epsilon", "Zeta",
    "Eta", "Theta", "Iota",
    "Kappa", "Lambda", "Mu",
    "Nu", "Xi", "Omicron",
    "Pi", "Rho", "Sigma",
    "Tau", "Upsilon", "Phi",
    "Chi", "Psi", "Omega",
];

/// Generates a random string.
fn gen_random_string() -> String {
    let mut rng = rand::thread_rng();
    let mut string = String::new();

    let len = rng.gen_range(4..=12);
    
    let mut x = 0;
    while x <= len {
        let choice = GREEK_ALPHABET.choose(&mut rng).unwrap();
        let choice = if string.len() != 0 { format!(" {}", choice) } else { choice.to_string() };
        string.push_str(&choice);
        x += 1;
    }

    string
}