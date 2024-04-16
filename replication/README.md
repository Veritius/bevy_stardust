# bevy_stardust_replicate
Server-authoritative game state replication for [bevy_stardust], using [serde] and [bincode].

## Features
- `Resource`, `Component`, and `Event` replication
- Special change tracking for replicated data
- Fine grained control over replication scope

[bevy_stardust]: https://github.com/Veritius/bevy_stardust/
[serde]: https://serde.rs/
[bincode]: https://docs.rs/bincode/latest/bincode/