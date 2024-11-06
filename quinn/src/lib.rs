#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

mod config;
mod plugin;
mod state;

pub use plugin::QuicPlugin;