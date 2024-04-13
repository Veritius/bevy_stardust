# bevy_stardust_replicate
Game state replication for [bevy_stardust](https://github.com/Veritius/bevy_stardust/).

This crate currently only supports server-authoritative (star model) applications.

## Features
- `Component` and `Resource` replication
- Controlled entirely with components
- Special change tracking for replicated data
- Fine grained control over replication scope