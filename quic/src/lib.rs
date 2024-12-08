#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

mod events;
mod futures;
mod logging;
mod taskpool;

pub mod connection;
pub mod endpoint;

pub use connection::{Connection, ConnectionState, ConnectError, ConnectionError};
pub use endpoint::{Endpoint, EndpointState, EndpointError};