//! A native UDP transport layer for bevy_stardust.

#![warn(missing_docs)]

pub mod connection;

#[cfg(feature="encryption")]
pub mod crypto;

mod reliability;