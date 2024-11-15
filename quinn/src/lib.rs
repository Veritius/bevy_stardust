#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

#[cfg(not(any(
    feature="async",
)))]
compile_error!("One of the following features must be enabled: async");

mod config;
mod plugin;
mod runtime;

pub use plugin::QuicPlugin;
pub use config::*;