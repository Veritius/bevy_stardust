pub mod protocol;

use bevy_stardust_shared::{bevy::prelude::*, protocol::ProtocolAppExts};

pub struct DemoSharedPlugin;
impl Plugin for DemoSharedPlugin {
    fn build(&self, app: &mut App) {
        // strs are used here, but you can use any type implementing Hash.
        // If you want to set your protocol ID directly, you can access the ProtocolBuilder resource.
        // 
        // If you change the protocol, either by changing your crates, or using plugins from other crates differently,
        // you should put a different value into this function. If you don't, outdated clients with different protocols can connect.
        app.gen_protocol_id(&[
            "bevy_stardust simpledemo",
            "0.0.1",
            // You can also use your shared crate's name and version with the env macro,
            // but it may change whenever you change the Cargo.toml (ie updating version)
            // env!("CARGO_PKG_NAME"),
            // env!("CARGO_PKG_VERSION"),
        ]);
    }
}