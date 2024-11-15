#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

#[cfg(not(any(
    feature="async",
)))]
compile_error!("One of the following features must be enabled: async");

mod commands;
mod config;
mod connection;
mod endpoint;
mod plugin;
mod runtime;

pub use plugin::QuicPlugin;
pub use config::*;