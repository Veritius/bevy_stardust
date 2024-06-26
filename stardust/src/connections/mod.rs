//! Virtual connections.
//! 
//! In Stardust, a virtual connection is any entity with the [`Peer`] component.
//! Additional functionality is given with other components, like [`PeerLifestage`].

mod peer;
mod security;

pub(crate) mod systems;

pub mod events;

pub use peer::*;
pub use security::*;