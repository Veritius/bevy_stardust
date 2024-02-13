# bevy_stardust_quic

[![License](https://img.shields.io/badge/license-MIT_or_Apache_2.0-green?color=green)](./)
[![Stardust version](https://img.shields.io/badge/bevy__stardust-0.3-blue?color=blue)](https://crates.io/crates/bevy_stardust)
[![Bevy version](https://img.shields.io/badge/bevy-0.12-blue?color=blue)](https://bevyengine.org/)
[![Crates.io](https://img.shields.io/crates/v/bevy_stardust_udp)](https://crates.io/crates/bevy_stardust)

A QUIC transport layer for [bevy_stardust](https://crates.io/crates/bevy_stardust) using [quinn-proto](https://github.com/quinn-rs/quinn).

## Features
- Fully synchronous and runs inside the Bevy scheduler
- Connection and message oriented API
- Reliability and ordering for messages
- Simple, easy to use API that abstracts the hard stuff
- Authentication and encryption with TLS
- Supports clients, servers, and listen servers

### Future features
- [ ] WASM support using WebTransport