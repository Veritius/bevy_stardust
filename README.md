<h1><p align="center">✨ bevy_stardust</p></h1>
Stardust is a networking crate built for, and tightly integrated into Bevy's architecture. Stardust intends to make networking easy, but lets you do the hard stuff when you want to.
<br></br>

![License badge](https://img.shields.io/github/license/veritius/bevy_stardust)
![Bevy version badge](https://img.shields.io/badge/bevy-0.11-blue?color=blue)

## Features
- Tightly integrated with Bevy ECS - everything is part of the `World` and `App`.
- Automatically compartmentalised network messages, separated into 'channels' defined with Bevy components.
- Multithreaded - Stardust network code is built off the back of Bevy's scheduler, so your systems run perfectly in parallel.
- Plugins can effortlessly add their own network code without any changes on your side.
- Use any transport method to send messages over the internet, including UDP, WebRTC, even [signal flags](https://en.wikipedia.org/wiki/International_maritime_signal_flags) - without changing any of your systems.
- Effortlessly write networked code - Stardust takes care of the hard stuff, so you can focus on the fun stuff.
- Replicate components, even those from other crates, with a single line of code.
- Control replication on a per-entity basis with components and bundles.

*Note: While you can use any transport method, Stardust by itself only supports native UDP.*

### Planned features
- Reliability
- Ordering
- Fragmentation
- Error checking
- Compression
- Encryption
- `bevy_mod_scripting` support