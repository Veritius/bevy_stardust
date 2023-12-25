//! # bevy_stardust_quic
//! A QUIC transport layer for bevy_stardust.

#![warn(missing_docs)]
#![feature(exclusive_wrapper)]

pub mod plugin;

mod events;
mod manager;
mod endpoint;

pub use manager::QuicManager;