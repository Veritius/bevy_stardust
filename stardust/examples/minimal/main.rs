mod client;
mod server;

use bevy::app::SubApp;
use bevy::prelude::*;
use bevy_stardust::shared::prelude::*;

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