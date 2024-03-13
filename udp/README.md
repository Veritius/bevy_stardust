# bevy_stardust_udp
A lightweight, highly customisable, native-UDP transport layer for [bevy_stardust](https://crates.io/crates/bevy_stardust) focused on good performance for real-time games.

The wire format of packets, especially that of the handshake, is liable to change dramatically between versions. The only guarantee of stability is that an older version will receive a correct error message when they're rejected during the handshake.

## Features
- Lightweight, simple, and friendly to use
- Minimal dependencies using feature flags
- Connection and message-oriented API
- Reliability, ordering, and fragmentation

### Future features
- Congestion control
- Encrypted and authenticated communications
    - Authentication using a remote authentication server
    - Authentication using TLS and X.509 certificates