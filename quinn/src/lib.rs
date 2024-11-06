#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

mod backend;
mod config;
mod connection;
mod endpoint;
mod plugin;

pub use plugin::QuicPlugin;