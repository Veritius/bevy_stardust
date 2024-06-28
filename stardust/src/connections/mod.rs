//! Virtual connections.
//! 
//! In Stardust, a virtual connection is represented as an entity
//! with the [`Peer`] component, referred to as a **peer entity**.
//! Peer entities don't do anything on their own. Instead, their
//! behavior is defined by new components.
//! 
//! # I/O and Messaging
//! Peers have an I/O abstraction called [`PeerMessages<D>`] that
//! acts as a queue of messages, bridging the gap between game systems
//! and transport layers. For example, `PeerMessages<Incoming>` is used
//! by transport layers to queue unread messages, which the application
//! and other plugins can use to read incoming messages.
//! 
//! For more information about messaging, see the [messages module](crate::messages).
//! 
//! # Additional Data
//! The `Peer` component by itself does not store much data.
//! Instead, that's left up to additional components.
//! Components that store peer-related data on peer entities
//! are prefixed with `Peer`, such as [`PeerUid`].

mod messages;
mod peer;
mod stats;

pub(crate) mod systems;
pub(crate) use messages::clear_message_queues_system;

pub mod events;

pub use messages::*;
pub use peer::*;
pub use stats::{PeerRtt, PeerStats};