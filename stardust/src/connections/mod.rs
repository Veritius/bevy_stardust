//! Virtual connections.
//! 
//! In Stardust, a virtual connection is any entity with the [`Peer`] component, referred to as a **peer entity**.
//! 
//! Any component that stores information specific to a peer entity
//! should have its name be prefixed with `Peer`, for example, [`PeerUid`].

mod peer;
mod security;

pub(crate) mod systems;

pub mod events;

pub use peer::*;
pub use security::*;