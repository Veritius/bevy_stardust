#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

mod endpoint;
mod plugin;

pub use endpoint::Endpoint;
pub use plugin::QuichePlugin;