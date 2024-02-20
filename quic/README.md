# bevy_stardust_quic

[![License](https://img.shields.io/badge/license-MIT_or_Apache_2.0-green?color=green)](./)
[![Stardust version](https://img.shields.io/badge/bevy__stardust-0.3-blue?color=blue)](https://crates.io/crates/bevy_stardust)
[![Bevy version](https://img.shields.io/badge/bevy-0.13-blue?color=blue)](https://bevyengine.org/)
[![Crates.io](https://img.shields.io/crates/v/bevy_stardust_quic)](https://crates.io/crates/bevy_stardust_quic)

A QUIC transport layer for [bevy_stardust](https://crates.io/crates/bevy_stardust) using [quinn-proto](https://github.com/quinn-rs/quinn).

## Features
- Fully synchronous and runs inside the Bevy scheduler
- Connection and message oriented API using framing
- Reliability and ordering for Stardust messages
- Simple, easy to use API that abstracts the hard stuff
- Secure-by-default authentication and encryption with TLS
- Supports clients, servers, and listen servers

## Usage
Adding the plugin:
```rust
use bevy::prelude::*;
use bevy_stardust::prelude::*;
use bevy_stardust_quic::*;

fn main() {
    let mut app = App::new();
    app.add_plugin((DefaultPlugins, StardustPlugin));
    app.add_plugin(QuicTransportPlugin {
        authentication: TlsAuthentication::Secure,
        reliable_streams: 32,
        timeout_delay: 30,
    });

    app.run();
}
```

Opening an endpoint:
```rust
fn open_client_endpoint_system(
    mut manager: QuicConnectionManager,
) {
    manager.open_client_endpoint(
        "localhost:0",
        Arc::new(/* Root certificates */),
    ).unwrap();
}

fn open_server_endpoint_system(
    mut manager: QuicConnectionManager,
) {
    manager.open_server_endpoint(
        "localhost:12345",
        Arc::new(/* Root certificates */),
        vec![/* End-entity and intermediate certificates */],
        // Private key
    ).unwrap();
}
```

Joining a remote connection:
```rust
fn join_server_system(
    mut manager: QuicConnectionManager
) {
    manager.try_connect(
        /* Endpoint entity ID goes here */,
        "localhost:12345",
        /* Server alt name goes here */,
    ).unwrap();
}
```