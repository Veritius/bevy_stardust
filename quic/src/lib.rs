//! # bevy_stardust_quic
//! A QUIC transport layer for bevy_stardust.

#![warn(missing_docs)]
#![feature(exclusive_wrapper)]

mod connections;
mod endpoints;
mod incoming;
mod outgoing;
mod misc;
mod plugin;

pub use plugin::QuicTransportPlugin;
pub use endpoints::{QuicEndpoint, QuicConnectionManager};
pub use connections::QuicConnection;