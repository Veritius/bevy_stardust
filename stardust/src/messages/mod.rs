//! This module exposes APIs for working with messages.
//! 
//! Messages are represented with the [`Message`] type, which is heap-allocated and cheaply clonable.
//! This is the smallest unit of information transmission between [peers] used by Stardust.
//! The contents of a message can be absolutely anything, from chat messages to game state.
//! Stardust will never do anything with the contents of your messages - that's up to your systems.
//! 
//! Messages are, in reality, just an abstraction. The nuts and bolts of how the messages
//! are actually exchanged between computers is entirely up to installed transport layers.
//! A message may be sent via datagrams, a byte stream, or something else entirely. You as
//! the developer don't need to worry about what's going on behind the scenes, because it
//! should just work.
//! 
//! You will primarily handle messages through [`PeerMessages<D>`]. This component is attached
//! to [peer entities][peers], and acts as a queue for incoming and outgoing messages, depending
//! on the choice of `D`. The documentation for `PeerMessages` goes into much further detail about
//! how you use these message queues.
//! 
//! [peers]: crate::connections
//! [`PeerMessages<D>`]: crate::connections::PeerMessages

mod channels;
mod constraints;
mod direction;
mod message;
mod queue;

// Re-exports
pub use bytes;

// Public types
pub use channels::{MessageChannel, MessageChannelId};
pub use constraints::MessageConsistency;
pub use direction::{NetDirection, MessageDirection, Incoming, Outgoing};
pub use message::{Message, ChannelMessage};
pub use queue::{MessageQueue, ChannelIter, MessageIter};