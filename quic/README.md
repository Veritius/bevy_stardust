# bevy_stardust_quic
A [sans-IO] protocol that sits between `bevy-stardust` and QUIC implementations, facilitating the exchange of Stardust messages. This protocol works regardless of the QUIC implementation.

Since this crate does not implement QUIC, it's useless by itself. Instead, it's intended to be used by other crates as a dependency, which will deal with I/O and the QUIC protocol.

[sans-IO]: https://sans-io.readthedocs.io/how-to-sans-io.html