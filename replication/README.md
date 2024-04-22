# bevy_stardust_replicate
Server-authoritative game state replication for [bevy_stardust].

## Features
- `Resource`, `Component`, and `Event` replication
- Not reliant on any existing serialisation crate
- Optional support for [serde] trait implementors

[bevy_stardust]: https://github.com/Veritius/bevy_stardust/
[serde]: https://serde.rs/
[bincode]: https://docs.rs/bincode/latest/bincode/