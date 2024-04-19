# bevy_stardust_udp
A lightweight, highly customisable, native-UDP transport layer for [bevy_stardust](https://crates.io/crates/bevy_stardust) focused on good performance for real-time games.

**Warning:** *Version 0.1.0 is missing several crucial features, but is in a good enough state for prototyping apps.*

## Features
- Lightweight, simple, and friendly to use
- Minimal dependencies using feature flags
- Connection and message-oriented API
- Reliable and ordered messages

### Unimplemented but planned
These features are unimplemented in this version, but will be added soon.
- Message windows
- Congestion control
- Message fragmentation

### Future features
- Encrypted and authenticated communications
    - Authentication using a remote authentication server
    - Authentication using TLS and X.509 certificates