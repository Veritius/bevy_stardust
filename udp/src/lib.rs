//! A native UDP transport layer for bevy_stardust.

#![warn(missing_docs)]

mod plugin;
mod manager;
mod endpoint;
mod connection;
mod sequences;
mod reliability;

pub use plugin::UdpTransportPlugin;
pub use manager::UdpManager;
pub use endpoint::Endpoint;
pub use connection::Connection;