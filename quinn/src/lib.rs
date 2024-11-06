#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

mod config;
mod frontend;
mod plugin;
mod state;

#[cfg(feature="async")]
mod backend_async;

pub use plugin::QuicPlugin;