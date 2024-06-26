//! This module exposes APIs for working with messages and channels.
//! 
//! ## Messages
//! Messages are individual, contiguous octet (byte) strings.
//! They are most often utilised with the [`Message`] type,
//! which provides various guarantees to the user about the data.
//! 
//! Messages are an abstraction over I/O, and are not a unit of transport.
//! How messages are actually exchanged between machines is up to the transport layer.
//! 
//! ## Channels
//! Channels are a way to differentiate the purpose of messages.
//! A message may be sent on a channel, which must be registered in the `App`.
//! 
//! Each channel has its own [configuration](ChannelConfiguration) value, stored in the [registry](ChannelRegistry).
//! This is used by the [transport layers](#transport) to efficiently handle message exchange.
//! These configuration values are also an optimisation method that you, the developer, can use to make better netcode.

pub mod channels;

mod direction;
mod message;
mod queue;

// Internal types
pub(crate) use queue::clear_message_queue_system;
pub(crate) use channels::{ChannelRegistryMut, ChannelRegistryInner};

// Re-exports
pub use bytes;

// Public types
pub use channels::{Channel, ChannelId, ChannelRegistry};
pub use direction::{NetDirection, NetDirectionType, Incoming, Outgoing};
pub use message::{Message, ChannelMessage};
pub use queue::{NetworkMessages, ChannelIter, MessageIter};