//! This module exposes APIs for working with messages and channels.
//! 
//! ## Messages
//! Messages are individual, contiguous octet (byte) strings, divided into channels.
//! In this sense, Stardust is a message-based system, not a stream-based one.
//! However, just like how TCP/IP and QUIC work, you can implement your own byte streaming.
//! 
//! An individual message has the following guarantees:
//! - Complete: messages are never received piecemeal.
//! - Unmodified: a received message is exactly what was sent.
//! 
//! Stardust by itself does not do any I/O operations, and is not responsible for upholding these guarantees.
//! 
//! ## Channels
//! Channels are a way to differentiate the purpose of messages.
//! A message may be sent on a channel, which must be registered in the `App`.
//! 
//! Each channel has its own [configuration](ChannelConfiguration) value, stored in the [registry](ChannelRegistry).
//! This is used by the [transport layers](#transport) to efficiently handle message exchange.
//! These configuration values are also an optimisation method that you, the developer, can use to make better netcode.

mod queue;
mod direction;
mod channels;

// Internal types
pub(crate) use queue::clear_message_queue_system;
pub(crate) use channels::{ChannelRegistryMut, ChannelRegistryInner};

// Re-exports
pub use bytes;

// Public types
pub use queue::{NetworkMessages, ChannelIter, MessageIter};
pub use direction::{NetDirection, NetDirectionType, Incoming, Outgoing};
pub use channels::{Channel, ChannelId, ChannelConfiguration, ChannelConsistency, ChannelData, ChannelRegistry, ChannelSetupAppExt};