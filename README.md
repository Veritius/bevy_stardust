<h1><p align="center">✨ bevy_stardust</p></h1>
Stardust is an opinionated networking crate built for Bevy, with a focus on extensibility and parallelism.
<br></br>

[![License](https://img.shields.io/github/license/veritius/bevy_stardust)](./)
[![Bevy version](https://img.shields.io/badge/bevy-0.12-blue?color=blue)](https://bevyengine.org/)
[![Crates.io](https://img.shields.io/crates/v/bevy_stardust)](https://crates.io/crates/bevy_stardust)

## Features
- Tightly integrated with Bevy ECS - everything is part of the `World` and `App`, using the scheduler for parallel network code, even in your game systems.
- Architecture agnostic - use client/server, peer to peer, mesh networks, you name it.
- Send data any way you want, over UDP, QUIC, WebRTC, WebSockets, TCP, HTTP - you don't even need to use the Internet: use AM radio or even [maritime signal flags](https://en.wikipedia.org/wiki/International_maritime_signal_flags) if you really want to.
- Write the same code no matter the transport layer you use.
- Full, flexible support for network-enabled plugins.

### Planned features
- Additional features for the UDP transport layer
    - Fragmentation and compression
    - Encryption and authentication
- Replication and state synchronisation API
- WebRTC and QUIC transport layers

## Usage
| Bevy | Stardust | UDP transport |
| ---- | -------- | ------------- |
| 0.12 | 0.2      | 0.1           |
| 0.11 | 0.1      | N/A           |

```rs
// Define your channels
#[derive(TypePath)]
struct MyChannel;

// Add it to your app
app.register_channel::<MyChannel>(ChannelConfiguration {
    reliable: ChannelReliability::SemiReliabile,
    ordered: false,
    fragmented: false,
    string_size: 10..=100,
});

// Read and write messages in game systems
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

Detailed information is available at [docs/usage.md](./docs/usage.md).

## Performance
**TODO: Performance comparisons**