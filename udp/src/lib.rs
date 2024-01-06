//! A native UDP transport layer for bevy_stardust.

#![warn(missing_docs)]

mod config;
mod connection;
mod handshake;
mod plugin;
mod reliability;
mod systems;

pub use plugin::UdpTransportPlugin;