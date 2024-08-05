# bevy_stardust_quinn
A transport layer for `bevy_stardust` based on `bevy_stardust_quic` and `quinn`. The API is synchronous, but I/O and QUIC packet handling runs asynchronously.