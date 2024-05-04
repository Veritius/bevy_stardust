#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

mod appdata;
mod connection;
mod endpoint;
mod manager;
mod plugin;
mod receiving;
mod sending;
mod sequences;
mod varint;

pub mod diagnostics;

/// Common imports.
/// 
/// `use bevy_stardust_udp::prelude::*;`
pub mod prelude {
    use super::*;
    pub use plugin::UdpTransportPlugin;
    pub use appdata::ApplicationNetworkVersion;
    pub use manager::{UdpManager, Unspecified};
    pub use endpoint::{Endpoint, EndpointState, statistics::EndpointStatistics};
    pub use connection::{Connection, ConnectionDirection, ConnectionState, statistics::ConnectionStatistics};
}