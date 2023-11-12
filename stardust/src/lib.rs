//! # bevy_stardust
//! A networking crate for the Bevy game engine.

#![warn(missing_docs)]

pub mod prelude;
pub mod connections;
pub mod protocol;
pub mod octets;
pub mod channels;
pub mod scheduling;
pub mod transports;
pub mod messages;

mod setup;