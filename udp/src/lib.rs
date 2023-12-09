//! A transport layer for bevy_stardust, using native UDP sockets.

#![warn(missing_docs)]

pub mod prelude;
pub mod policy;

mod established;
mod ordering;
mod plugin;
mod ports;
mod receiving;
mod reliability;
mod sending;

/// Information about the packet at the header of the packet.
const PACKET_HEADER_SIZE: usize = 3;
/// The maximum amount of bytes that can be stored in a single UDP packet's payload.
const MAXIMUM_TRANSPORT_UNITS: usize = 1472;