<h1><p align="center">✨ bevy_stardust</p></h1>
Stardust is a flexible networking crate built for Bevy, with a focus on extensibility and parallelism.
<br></br>

![License](https://img.shields.io/badge/license-MIT_or_Apache_2.0-green)
[![Bevy version](https://img.shields.io/badge/bevy-0.13-blue?color=blue)](https://bevyengine.org/)
[![Crates.io](https://img.shields.io/crates/v/bevy_stardust)](https://crates.io/crates/bevy_stardust)

## Why Stardust?
### ECS-integrated
Stardust is, simply put, just another plugin. All state information is in the Bevy `World` as entities, components, and systems. Connections are just entities, and you can attach any data you want to them without complex associative arrays, and access them in simple queries.

Stardust gives first class support to plugins, ensuring all APIs can be used by plugins without issue, and providing powerful organisation tools and abstractions.

### Parallel
Stardust is made specifically to run in parallel as well as possible. Since everything is in the ECS, the Bevy scheduler lets you run your networked game systems in parallel, with very little effort on your part.

The message queue APIs are made to be as parallel as possible, simply being components attached to connection entities. This means you can apply query filters to your heart's content, letting disjoint accesses perform network operations in parallel!

### Modular
Rather than being a monolith, the core of Stardust is simple: provide an API for sending and reading bytes, and an API for managing connections.

You can use any transport layer you want. Use UDP, TCP, QUIC, HTTP, some homebrew transport layer, I2C, AM radio, or even [maritime signal flags](https://en.wikipedia.org/wiki/International_maritime_signal_flags), all at the same time, with no extra effort. Crossplay has never been easier or more flexible.

You can use any replication or extra features you want. If you prefer a specific crate for replication, it's really easy to integrate it into Stardust, as long as it has some kind of API for taking in and outputting bytes.

## Planned extensions
The following features are planned to be created as additional crates, as part of the overall project.

- Replication plugin
- UDP, QUIC, and WebTransport plugins
- Real time voice plugin

## Usage
| Bevy | Stardust |
| ---- | -------- |
| 0.13 | 0.4      |
| 0.12 | 0.2      |
| 0.11 | 0.1      |

<br>

`bevy_stardust` is the core 'interface' crate. It provides everything you need to write netcode, but doesn't deal with Internet communication or things like replication - that's left up to other crates.


**A simple example project:**
```rust
// This example assumes that you don't have the reflect feature flag.
// If you do, make sure your channel types implement TypePath.
// Additionally, spawning NetworkPeer entities is handled by transport layer plugins.
// For the purpose of this example, we'll assume they magically appeared somehow.

use std::any::TypeId;
use bevy_ecs::prelude::*;
use bevy_app::{prelude::*, ScheduleRunnerPlugin, MainSchedulePlugin};
use bevy_stardust::prelude::*;

// Channels are accessed with types in the type system.
// Simply put, you just need to create simple types like this.
// You can use Rust's privacy system to control channel access.
struct MyChannel;

fn main() {
    let mut app = App::new();

    // At the very least, Stardust needs the MainSchedulePlugin to work.
    app.add_plugins((
        ScheduleRunnerPlugin::default(),
        StardustPlugin,
    ));

    // Each channel needs to be added (or 'registered') to the app.
    // Once you do this, it becomes visible in the ChannelRegistry.
    // The ChannelRegistry is effectively a giant table of every registered channel.
    app.add_channel::<MyChannel>(ChannelConfiguration {
        // 'Reliable' messages will be detected if lost.
        reliable: ReliabilityGuarantee::Reliable,

        // 'Ordered' messages will be received in the same order they're sent.
        ordered: OrderingGuarantee::Ordered,

        // 'Fragmentable' messages will be broken up for transmission if need be.
        // This is actually just a flag to say that the messages *might* need to be fragmented.
        // Whether or not things are fragmented is up to the transport layer.
        fragmented: true,

        // Higher priority messages will be sent before others.
        priority: 0,
    });

    // Any transport layers should be added after you register all channels.
    // This is just a rule of thumb, though, some might not need to be.
    // Make sure to check the relevant documentation.

    // Your systems can be added at any point, but we'll do them here.
    // Also see the scheduling types in the scheduling module for advanced usage.
    // Most of the time, you just need to put things in the update schedule.
    // Also, note that since these systems have disjoint accesses, they run in parallel.
    app.add_systems(Update, (send_words_system, read_words_system));
}

// Messages use the Bytes type.
// This is cheaply clonable and you can send the same message to multiple peers.
// For this example, we create one from the bytes of a static str.
const MESSAGE: Bytes = Bytes::from_static("Hello, world!".as_bytes());

// Queueing messages just requires component access.
// This means you can use query filters to achieve better parallelism.
fn send_words_system(
    registry: ChannelRegistry,
    mut query: Query<(Entity, &mut NetworkMessages<Outgoing>), With<NetworkPeer>>
) {
    // The ChannelId must be retrieved from the registry.
    // These are more friendly to store since they're just numbers.
    // You can cache them if you want, as long as they aren't used in different Worlds.
    let channel = registry.channel_id(TypeId::of::<MyChannel>()).unwrap();

    // You can also iterate in parallel, if you have a lot of things.
    for (entity, mut outgoing) in query.iter_mut() {
        // Bytes objects are cheaply clonable, reference counted storages.
        // You can send them to as many peers as you want once created.
        outgoing.push(channel, MESSAGE);
        println!("Sent a message to {entity:?}");
    }
}

// Reading messages also just requires component accesses.
// The reading queue is a different component from the sending queue.
// This means you can read and send bytes in parallel, or in different systems.
fn read_words_system(
    registry: ChannelRegistry,
    query: Query<(Entity, &NetworkMessages<Incoming>), With<NetworkPeer>>
) {
    let channel = registry.channel_id(TypeId::of::<MyChannel>()).unwrap();
    for (entity, incoming) in query.iter() {
        let messages = incoming.channel_queue(channel);
        for message in messages.iter() {
            // Stardust only outputs bytes, so you need to convert to the desired type.
            // Also, in real products, don't unwrap, write checks. Never trust user data.
            let string = std::str::from_utf8(&*message).unwrap();
            println!("Received a message from {entity:?}: {string:?}");
        }
    }
}
```