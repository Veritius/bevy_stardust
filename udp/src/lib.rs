//! A native UDP transport layer for bevy_stardust.

#![warn(missing_docs)]

pub mod plugin;
pub mod connection;

mod config;
mod reliability;