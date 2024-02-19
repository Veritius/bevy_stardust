<h1><p align="center">✨ bevy_stardust</p></h1>
Stardust is an opinionated networking crate built for Bevy, with a focus on extensibility and parallelism.
<br></br>

![License](https://img.shields.io/badge/license-MIT_or_Apache_2.0-green)
[![Bevy version](https://img.shields.io/badge/bevy-0.13-blue?color=blue)](https://bevyengine.org/)
[![Crates.io](https://img.shields.io/crates/v/bevy_stardust)](https://crates.io/crates/bevy_stardust)

## Features and design
Rather than being a monolith including everything you could possibly need, the core of Stardust is simply an API for sending and reading bytes, and managing connections.

Just like Bevy, if you don't like something, swap it out. If you're unsatisfied with your current replication plugin, you can choose a different one without rewriting all your network-related game systems for a new API.

Stardust is designed with this in mind, enabling the easy addition of Bevy plugins to the app that have full access to networking. Plugins and your very own game systems can be written once and apply to every situation effortlessly, without needing to know anything about how bytes are sent to and from connections.

This also means you can use multiple protocols for connections. Use a native UDP transport plugin for PC players, a QUIC transport plugin for web players, some kind of proprietary transport plugin for consoles, and even something that communicates using [maritime signal flags](https://en.wikipedia.org/wiki/International_maritime_signal_flags) - all at once, on the same game server. Crossplay has never been easier or more flexible.

Not that it needs to be a game server. The way Stardust is written, it has no concept of network topology, just connections. That means you can even use this plugin for P2P connections in a mesh topology, though make sure the plugins you use support that sort of thing.

### Planned features
- Replication and state synchronisation API
- UDP, QUIC, and WebRTC transport layers

## Usage
| Bevy | Stardust |
| ---- | -------- |
| 0.13 | 0.3      |
| 0.12 | 0.2      |
| 0.11 | 0.1      |

***

```rust
use bevy::prelude::*;
use bevy_stardust::prelude::*;

// First, create a type to reference your channel
// The TypePath trait is only necessary with the reflect feature flag
#[derive(TypePath)]
struct MyChannel;

// Set up your app
fn main() {
    // Create the app and add the plugin
    let mut app = App::new();
    app.add_plugins((DefaultPlugins, StardustPlugin));

    // Register the channel
    app.add_channel::<MyChannel>(ChannelConfiguration {
        reliable: ReliabilityGuarantee::Reliable,
        ordered: OrderingGuarantee::Ordered,
        fragmented: true,
        string_size: 0..=5,
    });

    // Any and all systems can send and receive messages
    app.add_systems(Update, my_system);
}

// Use game systems to read and write messages
fn my_system(
    mut writer: ChannelWriter<MyChannel>,
    reader: ChannelReader<MyChannel>,
) {
    // Sending a message is simple
    writer.send(Entity::PLACEHOLDER, "hello".into());

    // And it's not much effort to get a string back
    let read = reader.iter().next().unwrap().1;
    assert_eq!(std::str::from_utf8(&read).unwrap(), "hello");
}
```

You also need a transport layer to send your messages.