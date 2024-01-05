//! Channel configuration.
//! 
//! All settings are not definitive, but hints to transport layers as how to treat channels.

use std::{ops::RangeInclusive, hash::Hasher};
use crate::protocol::StableHash;

/// Configuration for a channel.
#[derive(Debug)]
pub struct ChannelConfiguration {
    /// Whether messages will be resent if they're missed.
    pub reliable: ReliabilityGuarantee,

    /// Whether messages should be read in the order they were sent.
    /// With reliability set on, this can cause delays in reading messages on the channel.
    pub ordered: OrderingGuarantee,

    /// If messages should be broken up to send.
    /// If disabled, messages over the MTU will be discarded or panic, depending on the transport layer.
    /// If enabled, each octet string will have a tiny bit more overhead.
    pub fragmented: bool,

    /// How long an octet string sent over this channel will be, used for optimisations.
    /// Octet strings with lengths outside this range may cause warnings or panics in transport layers.
    pub string_size: RangeInclusive<u32>,
}

impl StableHash for &ChannelConfiguration {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.reliable.hash(state);
        self.ordered.hash(state);
        self.fragmented.hash(state);
        self.string_size.start().hash(state);
        self.string_size.end().hash(state);
    }
}

/// The reliability guarantee of a channel.
#[derive(Debug)]
pub enum ReliabilityGuarantee {
    /// If a message is lost, it's lost. There will be no attempt to get it back.
    Unreliable,
    /// If a message is lost, it will be resent. This incurs some overhead.
    Reliable,
}

impl StableHash for ReliabilityGuarantee {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            ReliabilityGuarantee::Unreliable => state.write_u8(0),
            ReliabilityGuarantee::Reliable => state.write_u8(1),
        }
    }
}

/// The ordering guarantee of a channel.
#[derive(Debug)]
pub enum OrderingGuarantee {
    /// Messages will be read in the order they are received.
    Unordered,
    /// Messages that are out of order will be discarded.
    Sequenced,
    /// Messages will be reordered to be in the order they were sent.
    Ordered,
}

impl StableHash for OrderingGuarantee {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            OrderingGuarantee::Unordered => state.write_u8(0),
            OrderingGuarantee::Sequenced => state.write_u8(1),
            OrderingGuarantee::Ordered => state.write_u8(2),
        }
    }
}