# bevy_stardust_replicate
Authoritative game state replication for [bevy_stardust].

## Features
- `Resource`, `Component`, and `Event` replication
- Integration with Bevy's change detection
- Not reliant on any existing serialisation crate
- Optional support for using [serde] (with [bincode])

## Future features
- Delta encoding/compression

[bevy_stardust]: https://github.com/Veritius/bevy_stardust/
[serde]: https://serde.rs/
[bincode]: https://docs.rs/bincode/latest/bincode/