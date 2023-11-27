//! A transport layer for bevy_stardust, using native UDP sockets.

#![warn(missing_docs)]

mod plugin;
mod established;

pub use plugin::UdpTransportPlugin;
pub use established::UdpConnection;