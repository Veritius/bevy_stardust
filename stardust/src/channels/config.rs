//! Channel configuration.
//! 
//! All settings are not definitive, but hints to transport layers as how to treat channels.

#[cfg(feature="hashing")]
use {std::hash::Hasher, crate::hashing::StableHash};

/// Configuration for a channel.
#[derive(Debug, Clone)]
pub struct ChannelConfiguration {
    /// Whether messages should be resent if they're missed.
    pub reliable: ReliabilityGuarantee,

    /// Whether messages should be read in the order they were sent.
    pub ordered: OrderingGuarantee,

    /// If messages on this channel may need to be broken up to be transmitted.
    /// If disabled, messages over the MTU will be discarded or panic, depending on the transport layer.
    pub fragmented: bool,

    /// The priority of messages on this channel.
    /// Transport values will send messages on channels with higher `priority` values first.
    /// Channel priority is not hashed when the `hashing` feature is enabled.
    pub priority: u32,
}

#[cfg(feature="hashing")]
impl StableHash for &ChannelConfiguration {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.reliable.hash(state);
        self.ordered.hash(state);
        self.fragmented.hash(state);
    }
}

/// The reliability guarantee of a channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReliabilityGuarantee {
    /// Messages are not guaranteed to arrive.
    Unreliable,

    /// Lost messages will be detected and resent.
    Reliable,
}

#[cfg(feature="hashing")]
impl StableHash for ReliabilityGuarantee {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            ReliabilityGuarantee::Unreliable => state.write_u8(0),
            ReliabilityGuarantee::Reliable => state.write_u8(1),
        }
    }
}

/// The ordering guarantee of a channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderingGuarantee {
    /// Messages will be available in the order they are received.
    /// This is not necessarily the order they were sent. If that matters, use a different variant.
    Unordered,

    /// Messages that are older than the most recent value will be discarded.
    /// Therefore, messages will be available in order, but out of order messages are lost.
    Sequenced,

    /// Messages will be available in the exact order they were sent.
    /// If [reliability](ReliabilityGuarantee::Reliable) is used, this can 'block' messages temporarily due to data loss.
    Ordered,
}

#[cfg(feature="hashing")]
impl StableHash for OrderingGuarantee {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            OrderingGuarantee::Unordered => state.write_u8(0),
            OrderingGuarantee::Sequenced => state.write_u8(1),
            OrderingGuarantee::Ordered => state.write_u8(2),
        }
    }
}