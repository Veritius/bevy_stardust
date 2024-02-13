# bevy_stardust_quic

[![License](https://img.shields.io/badge/license-MIT_or_Apache_2.0-green?color=green)](./)
[![Stardust version](https://img.shields.io/badge/bevy__stardust-0.3-blue?color=blue)](https://crates.io/crates/bevy_stardust)
[![Bevy version](https://img.shields.io/badge/bevy-0.12-blue?color=blue)](https://bevyengine.org/)
[![Crates.io](https://img.shields.io/crates/v/bevy_stardust_udp)](https://crates.io/crates/bevy_stardust)

A synchronous QUIC transport layer for [bevy_stardust](https://crates.io/crates/bevy_stardust) using [quinn-proto](https://github.com/quinn-rs/quinn). All logic is run within the Bevy scheduler in game systems, preventing any weird work-stealing related problems.

At some point in the future, this crate will support WASM using [WebTransport](https://developer.chrome.com/docs/capabilities/web-apis/webtransport). That isn't now, though. Feel free to contribute if you want it in the crate :)