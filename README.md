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

## Planned features
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