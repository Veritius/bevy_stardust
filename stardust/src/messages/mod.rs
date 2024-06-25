//! Message related stuff.

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