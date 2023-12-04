//! Channel configuration.
//! 
//! All settings are not definitive, but hints to transport layers as how to treat channels.

use std::ops::Range;

/// Configuration for a channel.
#[derive(Debug, Hash)]
pub struct ChannelConfiguration {
    /// See [ChannelReliability]'s documentation.
    pub reliable: ChannelReliability,
    /// See [ChannelOrdering]'s documentation.
    pub ordering: ChannelOrdering,
    /// See [ChannelFragmentation]'s documentation.
    pub fragment: ChannelFragmentation,

    /// How long an octet string sent over this channel will be, on average.
    pub expected_string_size: Range<u32>,
}

/// If a packet is missed, it will be resent. This can take a (relatively) long time.
/// If used with [ordering](ChannelOrdering::Ordered) this can cause [head-of-line blocking].
/// 
/// [head-of-line blocking]: https://en.wikipedia.org/wiki/Head-of-line_blocking
#[derive(Debug, Hash)]
pub enum ChannelReliability {
    /// Messages will not be resent if missed.
    Unreliable,
    /// Messages will be resent if missed.
    Reliable,
}

/// Ensure that systems read octet strings in the exact order they were sent over the wire.
/// Internet infrastructure doesn't guarantee the order of arrival, so it must be dealt with by the software.
#[derive(Debug, Hash)]
pub enum ChannelOrdering {
    /// Messages will be read in the order they arrive.
    Unordered,
    /// Messages will be read in the order they were sent.
    Ordered,
}

/// If a message is too large to send in a single packet, it'll instead be sent in multiple pieces, and recombined later.
/// It's highly recommended to use [reliability](ChannelReliability::Reliable), since most transport layers will discard the entire thing if one packet is missed.
#[derive(Debug, Hash)]
pub enum ChannelFragmentation {
    /// Disable fragmentation.
    Disabled,
    /// Enable fragmentation.
    Enabled,
}