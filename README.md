<h1><p align="center">✨ bevy_stardust</p></h1>
Stardust is an opinionated networking crate built for Bevy, with a focus on extensibility and parallelism.
<br></br>

[![License](https://img.shields.io/badge/license-MIT_or_Apache_2.0-green?color=green)](./)
[![Bevy version](https://img.shields.io/badge/bevy-0.12-blue?color=blue)](https://bevyengine.org/)
[![Crates.io](https://img.shields.io/crates/v/bevy_stardust)](https://crates.io/crates/bevy_stardust)

## Features
- Message and connection-oriented, with support for reliable and unreliable messages
- Tightly integrated with Bevy ECS - everything is part of the `World` and `App`, using the scheduler for parallel network code, even in your game systems.
- Architecture agnostic - use client/server, peer to peer, mesh networks, you name it.
- Send data any way you want, over UDP, QUIC, WebRTC, WebSockets, TCP, HTTP - you don't even need to use the Internet: use AM radio or even [maritime signal flags](https://en.wikipedia.org/wiki/International_maritime_signal_flags) if you really want to.
- Write the same code no matter the transport layer you use.
- Full, flexible support for network-enabled plugins.

### Planned features
- Replication and state synchronisation plugin
- UDP, QUIC, and WebRTC transport layers

## Usage
| Bevy | Stardust | UDP transport |
| ---- | -------- | ------------- |
| 0.12 | 0.2      | N/A           |
| 0.11 | 0.1      | Included      |

***

```rust
use bevy::prelude::*;
use bevy_stardust::prelude::*;

// First, define a channel type
#[derive(TypePath)]
struct MyChannel;

// Set up your app
fn main() {
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

// A simple system to read and write messages
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