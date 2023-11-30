//! A transport layer for bevy_stardust, using native UDP sockets.

#![warn(missing_docs)]

pub mod prelude;
pub mod policy;

mod established;
mod plugin;
mod ports;
mod receiving;
mod sending;

/// The maximum amount of bytes that can be stored in a single UDP packet's payload.
const MAXIMUM_PACKET_LENGTH: usize = 1472;