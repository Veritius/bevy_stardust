<h1><p align="center">✨ bevy_stardust</p></h1>
Stardust is a batteries-included networking crate built for Bevy. Stardust intends to make networking easy, but lets you do the hard stuff when you want to.
<br></br>

![License badge](https://img.shields.io/github/license/veritius/bevy_stardust)
![Bevy version badge](https://img.shields.io/badge/bevy-0.11-blue?color=blue)


## Features
- Tightly integrated with Bevy ECS - everything is part of the `World` and `App`.
- Effortlessly write networked code as part of your regular Bevy systems.
- Automatically compartmentalised network messages, separated into 'channels' defined with Bevy components.
- Runs in parallel - Stardust network code is built off the back of Bevy's scheduler, so your systems run perfectly in parallel.
- Plugins can effortlessly add their own network code without any changes on your side.
- Use any transport layer to send messages over the internet, including UDP, WebRTC, even [signal flags](https://en.wikipedia.org/wiki/International_maritime_signal_flags) - without changing any of your systems.
- Replicate components, even those from other crates, with a single line of code.
- Control replication on a per-entity basis with components and bundles.

*Note: While you can use any transport layer, Stardust by itself only supports native UDP.*

### Planned features
- Reliability
- Ordering
- Fragmentation
- Error checking
- Compression
- Encryption
- Randomness
- Replication
- `bevy_mod_scripting` support
