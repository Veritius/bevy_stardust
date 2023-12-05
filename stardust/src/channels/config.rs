//! Channel configuration.
//! 
//! All settings are not definitive, but hints to transport layers as how to treat channels.

use std::ops::Range;

/// Configuration for a channel.
#[derive(Debug, Hash)]
pub struct ChannelConfiguration {
    /// See [ChannelReliability]'s documentation.
    pub reliable: bool,
    /// See [ChannelOrdering]'s documentation.
    pub ordered: bool,
    /// See [ChannelFragmentation]'s documentation.
    pub fragmented: bool,

    /// How long an octet string sent over this channel will be, used for optimisations.
    /// Octet strings with lengths outside this range may cause panics in transport layers.
    pub expected_string_size: Range<u32>,
}