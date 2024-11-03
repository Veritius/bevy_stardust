#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

mod backend;
mod config;
mod ecs;
mod plugin;

pub use plugin::QuicPlugin;
pub use ecs::connection::Connection;
pub use ecs::endpoint::Endpoint;
pub use ecs::params::QuicManager;