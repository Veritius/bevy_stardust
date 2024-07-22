#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

// #[cfg(not(any(feature="quiche")))]
// compile_error!("You must choose a QUIC implementation.");

mod bimap;
mod config;
mod connection;
mod endpoint;
mod events;
mod plugin;

pub use config::{AppProtosBuilder, AppProtos, AppProto};
pub use connection::Connection;
pub use endpoint::Endpoint;
pub use events::TryConnectEvent;
pub use plugin::QuicPlugin;