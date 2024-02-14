//! # bevy_stardust
//! A networking crate for the Bevy game engine.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod channels;
pub mod connections;
pub mod plugin;
pub mod prelude;
pub mod scheduling;

#[cfg(feature="hashing")]
pub mod hashing;