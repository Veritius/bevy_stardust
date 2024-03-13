//! A native UDP transport layer for bevy_stardust.

#![warn(missing_docs)]

mod connection;
mod endpoint;
mod manager;
mod packet;
mod plugin;
mod receiving;
mod sending;
mod sequences;

pub use plugin::UdpTransportPlugin;
pub use manager::UdpManager;
pub use endpoint::{Endpoint, EndpointState, EndpointStatistics};
pub use connection::{Connection, ConnectionDirection, ConnectionState, statistics::ConnectionStatistics};