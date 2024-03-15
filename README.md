<h1><p align="center">✨ bevy_stardust</p></h1>
Stardust is a flexible networking crate built for Bevy, with a focus on extensibility and parallelism.
<br></br>

![License](https://img.shields.io/badge/license-MIT_or_Apache_2.0-green)
[![Bevy version](https://img.shields.io/badge/bevy-0.13-blue?color=blue)](https://bevyengine.org/)
[![Crates.io](https://img.shields.io/crates/v/bevy_stardust)](https://crates.io/crates/bevy_stardust)

## Why Stardust?
### Bevy-oriented and parallel
Stardust is made with Bevy, for Bevy. Everything can be modified and controlled by systems. All data is stored in the ECS. Everything runs in parallel, writing bytes, receiving bytes, controlled by the scheduler using `SystemParam` implementations.

### Simple
Want to send something?
```rust
// Messages are distinguished with the type system
fn my_writer_system(mut writer: NetworkWriter<MyMessage>) {
    writer.send(
        // Connections are entities, just use their ID!
        Entity::PLACEHOLDER,
        // Send anything you can turn into bytes
        b"Hello, world!".into()
    );
}
```

Want to receive something?
```rust
// Reading messages is not blocking
fn my_reader_system(reader: NetworkReader<MyMessage>) {
    let v = reader.iter().next().unwrap().1;
    assert_eq!(b"Hello, world!", &v);
}
```

Want to close a connection?
```rust
// Because you use queries, you can filter your accesses
fn my_disconnection_system(mut connections: Query<&mut NetworkPeer>) {
    for connection in connections.iter_mut() {
        connection.disconnect();
    }
}
```

### Modular
Rather than being a monolith, the core of Stardust is simple: provide an API for sending and reading bytes, and an API for managing connections.

You can use any transport layer you want, and it just works. Use UDP, TCP, QUIC, HTTP, some homebrew transport layer, I2C, AM radio, or even [maritime signal flags](https://en.wikipedia.org/wiki/International_maritime_signal_flags), anything you want - all at the same time, with no extra effort. Crossplay has never been easier or more flexible.

You can use any replication or extra features you want. If you prefer a specific crate for replication, it's really easy to integrate it into Stardust, as long as it has some kind of API for taking in and outputting bytes. It'll just work.

### Planned features
The following features are planned to be created as additional crates.

- Replication plugin
- UDP, QUIC, and WebTransport plugins
- Real time voice plugin

## Usage
| Bevy | Stardust |
| ---- | -------- |
| 0.13 | 0.4      |
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
    writer.send(Entity::PLACEHOLDER, b"Hello, world!".into());

    // And it's not much effort to get a string back
    let read = reader.iter().next().unwrap().1;
    assert_eq!(b"Hello, world!", &read);
}
```

Note that your messages will **not be sent** without a transport plugin.