//! Virtual connections.
//! 
//! In Stardust, a virtual connection is represented as an entity
//! with the [`Peer`] component, referred to as a **peer entity**.
//! Peer entities don't do anything on their own.
//! 
//! Any component that stores information specific to a peer entity should have
//! its name be prefixed with `Peer`, for example, [`PeerUid`] or [`PeerMessages`].

mod messages;
mod peer;
mod security;

pub(crate) mod systems;

pub mod events;

pub use messages::*;
pub use peer::*;
pub use security::*;