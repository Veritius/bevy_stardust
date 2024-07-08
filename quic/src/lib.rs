#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

#[cfg(not(any(feature="quiche")))]
compile_error!("You must pick a QUIC implementation.");

mod connection;
mod endpoint;
mod plugin;

pub use connection::Connection;
pub use endpoint::Endpoint;
pub use plugin::QuicPlugin;