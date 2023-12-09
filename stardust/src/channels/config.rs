//! Channel configuration.
//! 
//! All settings are not definitive, but hints to transport layers as how to treat channels.

use std::ops::RangeInclusive;

/// Configuration for a channel.
#[derive(Debug, Hash)]
pub struct ChannelConfiguration {
    /// Whether messages will be resent if they're missed.
    /// 
    /// `false` disables reliability. If a message is lost, it's lost. There will be no attempt to get it back.
    /// 
    /// `true` enables reliability. If a message is lost, it will be resent. This incurs some overhead.
    /// If a lot of packets are lost, messages from other channels can be blocked by attempts to recover lost information.
    pub reliable: bool,

    /// Whether messages should be read in the order they were sent.
    /// With reliability set on, this can cause delays in reading messages on the channel.
    pub ordered: bool,

    /// If messages should be broken up to send.
    /// If disabled, messages over the MTU will be discarded or panic, depending on the transport layer.
    /// If enabled, each octet string will have a tiny bit more overhead.
    pub fragmented: bool,

    /// How long an octet string sent over this channel will be, used for optimisations.
    /// Octet strings with lengths outside this range may cause warnings or panics in transport layers.
    pub string_size: RangeInclusive<u32>,
}