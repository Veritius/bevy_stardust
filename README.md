# bevy_stardust
A (very opinionated) experimental client/server networking crate for Bevy, fully operating within the ECS paradigm.

This crate is highly experimental and you shouldn't use it for anything you care about. Things *will* break.
This is also a hobby project. It's probably not very good, but you're more than welcome to help out!

**Cryptographic features are not currently supported. Man in the middle attacks are really, really easy with this crate.**

## Planned features
- Reliability
- Ordering
- Fragmentation
- Error checking
- Compression
- Encryption
- Signing

## Other crates

If you want peer-to-peer networking, use [bevy_ggrs](https://github.com/gschup/bevy_ggrs). If you want something with more granular control use [naia](https://github.com/naia-lib/naia).

Minimum supported Rust version is the latest stable release.

| Bevy version | Crate version |
| ------------ | ------------- |
| 0.11         | main          |
