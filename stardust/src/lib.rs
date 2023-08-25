//! # bevy_stardust
//! A networking crate for the Bevy game engine.

pub mod setup;
pub mod prelude;

pub mod client;
pub mod server;

pub mod protocol;
pub mod octets;
pub mod channels;
pub mod scheduling;
pub mod transports;