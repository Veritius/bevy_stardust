//! # bevy_stardust_quic
//! A QUIC transport layer for bevy_stardust.

#![warn(missing_docs)]
#![feature(exclusive_wrapper)]

mod connections;
mod endpoints;
mod incoming;
mod logging;
mod misc;
mod outgoing;
mod plugin;
mod polling;

pub use plugin::QuicTransportPlugin;
pub use endpoints::{QuicEndpoint, QuicConnectionManager};
pub use connections::QuicConnection;