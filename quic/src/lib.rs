#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

mod events;
mod futures;
mod logging;
mod plugin;
mod systems;
mod taskpool;

pub mod config;
pub mod connection;
pub mod endpoint;
pub mod utilities;

pub use connection::{Connection, ConnectionState, ConnectError, ConnectionError};
pub use endpoint::{Endpoint, EndpointWeak, EndpointState, EndpointError};
pub use plugin::QuicPlugin;
pub use taskpool::WorkerThreads;