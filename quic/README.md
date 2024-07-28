# bevy_stardust_quic
A [sans-IO] wrapper that implements a statemachine for QUIC communication in `bevy_stardust` applications.


This crate doesn't actually add a plugin or a QUIC implementation. Instead, it just sits between `bevy_stardust` and a QUIC implementation like `quinn` or `quiche`, dealing with streams and datagrams and such, and outputting Stardust messages for the application.

[sans-IO]: https://sans-io.readthedocs.io/how-to-sans-io.html