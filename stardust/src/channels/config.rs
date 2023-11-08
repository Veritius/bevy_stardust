//! Channel configuration.
//! 
//! All settings are not definitive, but hints to transport layers as how to treat channels.

use std::{any::TypeId, marker::PhantomData};
use super::id::ChannelId;

/// Configuration for a channel.
#[derive(Debug, Hash)]
pub struct ChannelConfiguration {
    /// See [ChannelReliability]'s documentation.
    pub reliable: ChannelReliability,
    /// See [ChannelOrdering]'s documentation.
    pub ordering: ChannelOrdering,
    /// See [ChannelFragmentation]'s documentation.
    pub fragment: ChannelFragmentation,
    /// See [ChannelCompression]'s documentation.
    pub compress: ChannelCompression,
    /// See [MessageValidation]'s documentation.
    pub validate: MessageValidation,
}

/// Immutable channel information, owned by the `ChannelRegistry`.
pub struct ChannelData {
    /// The channel's `TypeId`.
    pub type_id: TypeId,
    /// The channel's `TypePath` (from `bevy_reflect`)
    pub type_path: &'static str,
    /// The channel's sequential ID assigned by the registry.
    pub channel_id: ChannelId,

    /// The config of the channel.
    pub config: ChannelConfiguration,

    // Prevent this type being constructed
    pub(super) phantom: PhantomData<()>,
}

/// Reliable channels.
#[derive(Debug, Hash)]
pub enum ChannelReliability {
    /// Messages will not be resent if missed.
    Unreliable,
    /// Messages will be resent if missed.
    Reliable,
}

/// Ordered channels.
#[derive(Debug, Hash)]
pub enum ChannelOrdering {
    /// Messages will be read in the order they arrive.
    Unordered,
    /// Messages will be read in the order they were sent.
    Ordered,
}

/// Fragmented messages.
#[derive(Debug, Hash)]
pub enum ChannelFragmentation {
    /// Messages that are too large to send will not be sent.
    /// Additional behavior such as panicking depends on the transport layer.
    Disabled,
    /// Messages that are too large will be spread across multiple packets.
    /// It's highly recommended to use [reliability](ChannelReliability::Reliable) if you are using this.
    /// If a segment of a fragmented octet string is missed, the transport layer may discard the entire thing.
    Enabled,
}

/// Compresses octet strings on channels.
#[derive(Debug, Hash)]
pub enum ChannelCompression {
    /// Don't compress messages.
    Disabled,
    /// Compress messages, but don't sacrifice speed.
    Fast,
    /// Compress messages as much as possible.
    High,
}

/// Ensure messages are as they were sent.
#[derive(Debug, Hash)]
pub enum MessageValidation {
    /// The integrity of messages will not be checked.
    Disabled,
    /// The integrity of messages will be checked.
    Enabled,
}