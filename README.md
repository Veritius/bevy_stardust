<h1><p align="center">✨ bevy_stardust</p></h1>
Stardust is an opinionated, batteries-included networking crate built for Bevy, with a focus on extensibility and parallelism.
<br></br>

![License](https://img.shields.io/github/license/veritius/bevy_stardust)
[![Bevy version](https://img.shields.io/badge/bevy-0.11-blue?color=blue)](https://bevyengine.org/)
[![Crates.io](https://img.shields.io/crates/v/bevy_stardust)](https://crates.io/crates/bevy_stardust)

## Features
- Tightly integrated with Bevy ECS - everything is part of the `World` and `App`, using the scheduler for parallel network code, even in your game systems.
- Architecture agnostic - use client/server, peer to peer, mesh networks, you name it.
- Send data any way you want, over UDP, WebRTC, WebSockets - you don't even need to use the Internet: use AM radio or even [maritime signal flags](https://en.wikipedia.org/wiki/International_maritime_signal_flags) if you really want to.
- No matter the transport layer you use, write the same code in your game systems, entirely in parallel using Bevy's scheduler.
- Add network-enabled Bevy plugins with no extra effort on your end.

### Planned features
- Features for the UDP transport layer
    - Fragmentation
    - Error checking
    - Compression
    - Encryption (incl. authentication)
- Replication
- Included WebRTC transport layer
- `bevy_mod_scripting` support

## Usage
**TODO: Quick start guide**

Detailed information is available at [docs/usage.md](./docs/usage.md).

## Performance
**TODO: Performance comparisons**