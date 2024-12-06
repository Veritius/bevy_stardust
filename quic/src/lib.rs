#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

mod connection;
mod endpoint;
mod taskpool;

pub use connection::Connection;
pub use endpoint::Endpoint;