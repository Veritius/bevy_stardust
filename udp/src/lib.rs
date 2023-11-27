//! A transport layer for bevy_stardust, using native UDP sockets.

#![warn(missing_docs)]

mod established;
mod plugin;
mod ports;
mod receiving;
mod sending;

pub use established::UdpConnection;
pub use plugin::UdpTransportPlugin;