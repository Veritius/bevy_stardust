//! Channel configuration.
//! 
//! All settings are not definitive, but hints to transport layers as how to treat channels.

use std::ops::RangeInclusive;

/// Configuration for a channel.
#[derive(Debug, Hash)]
pub struct ChannelConfiguration {
    /// See [ChannelReliability]'s documentation.
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

/// How reliable a channel should be.
#[derive(Debug, Hash)]
pub enum ChannelReliability {
    /// No reliability. If the message is lost, it's lost. No attempt will be made to get it back.
    Unreliable,

    /// Some reliability. If the message is lost, an attempt will be made to get it resent.
    /// The transport layer won't drop everything to do so, however, so there is the potential for this channel to be lost.
    SemiReliable,

    /// Full reliability. If the message is lost, anything and everything will be done to get it back.
    /// Unlike `SemiReliable`, the message will _never_ be lost, at the cost of potentially blocking other `FullyReliable` messages.
    FullyReliable,
}