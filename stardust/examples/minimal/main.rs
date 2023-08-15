mod client;
mod server;

use bevy::app::SubApp;
use bevy::prelude::*;
use bevy_stardust::shared::prelude::*;
use rand::seq::SliceRandom;

fn main() {
    let mut owner = App::new();
    owner.add_plugins(DefaultPlugins);
    owner.set_runner(|mut app| loop { app.update() });

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
    app.register_channel::<RandomDataChannel>(ChannelConfig {
        direction: ChannelDirection::Bidirectional,
    }, ());
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
    
    let mut x = 0;
    while x <= 8 {
        let choice = GREEK_ALPHABET.choose(&mut rng).unwrap();
        let choice = if string.len() != 0 { format!(" {}", choice) } else { choice.to_string() };
        string.push_str(&choice);
        x += 1;
    }

    string
}