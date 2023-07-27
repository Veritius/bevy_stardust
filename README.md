<h1><p align="center">✨ bevy_stardust</p></h1>
Stardust is a networking crate built for, and tightly integrated into Bevy's architecture. Stardust intends to make networking easy, but lets you do the hard stuff when you want to.

## Features
- Enable replicating a component (even from another crate!) with only a few lines of code.
- Per-entity and per-component control of replication. Disable replicating a component as part of a bundle, or only allow it on some entities.
- Bevy plugins can easily add their own networked code.
- Easy organisation of network messages, automatically compartmentalised into 'channels'.
- Stardust's code is primarily server-authoritative, but you can ignore all of that and do determinism if you want.

### Planned features
- Reliability
- Ordering
- Fragmentation
- Error checking
- Compression
- Encryption

## Why you may not want Stardust
Stardust will never be as efficient as manually writing your own networking code. In fact, Stardust is pretty inefficient, with a lot of the code being more focused on developer ergonomics rather than being extremely performant.

Stardust is a hobby project, with no guarantee of anything. Internally, there are no guarantees of the consistency of the code. While the API surface will stay largely the same, there may be dramatic changes under the hood with little warning.

Additionally, Stardust is native-only, using UDP sockets. It does not support wasm, but this might change in future.

It's worth having a look at the [other options](https://bevyengine.org/assets/#networking) before committing to using Stardust.