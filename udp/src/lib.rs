//! A native UDP transport layer for bevy_stardust.

#![warn(missing_docs)]

mod connection;
mod handshake;
mod plugin;
mod reliability;
mod manager;
mod endpoint;

pub use plugin::UdpTransportPlugin;
pub use manager::UdpManager;
pub use endpoint::Endpoint;
pub use connection::Connection;