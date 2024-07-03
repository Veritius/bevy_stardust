#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

mod version;
mod connection;
mod endpoint;
mod manager;
mod plugin;
mod schedule;
mod sequences;

pub mod diagnostics;

/// Common imports.
/// 
/// `use bevy_stardust_udp::prelude::*;`
pub mod prelude {
    use super::*;
    pub use plugin::UdpTransportPlugin;
    pub use version::{AppVersion, DeniedMinorVersions};
    pub use manager::{UdpManager, Unspecified};
    pub use endpoint::{Endpoint, EndpointState, statistics::EndpointStatistics};
    pub use connection::{Connection, statistics::ConnectionStatistics};
}