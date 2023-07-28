use std::any::TypeId;
use bevy::prelude::*;
use super::id::ChannelId;

/// An entity representing a channel.
#[derive(Component)]
pub struct ChannelData {
    pub(crate) config: ChannelConfig,
    pub(crate) type_id: TypeId,
    pub(crate) channel_id: ChannelId,
}

impl ChannelData {
    /// Returns ChannelConfig.
    pub fn config(&self) -> ChannelConfig {
        self.config
    }

    /// Returns the associated `TypeId` used to access this channel.
    pub fn type_id(&self) -> TypeId {
        self.type_id
    }

    /// Returns the associated `ChannelId` used for network transport.
    pub fn channel_id(&self) -> ChannelId {
        self.channel_id
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChannelConfig {
    pub direction: ChannelDirection,
}

/// Configures the direction of a channel.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ChannelDirection {
    /// The client and server can both send messages on this channel.
    #[default]
    Bidirectional,
    /// Only the server can send messages on this channel.
    ServerToClient,
    /// Only a client can send a message on this channel.
    ClientToServer,
}

/// Marks this channel as ordered - messages sent in this channel will arrive in the exact order they are sent. Messages may not arrive, use [ReliableChannel] to ensure they do.
#[derive(Component, Default, Clone, Copy)]
pub struct OrderedChannel;

/// Marks this channel as reliable - messages sent in this channel are guaranteed to arrive eventually.
#[derive(Component, Default, Clone, Copy)]
pub struct ReliableChannel;

/// Discards packets in this channel that are older than a certain amount of ticks.
#[derive(Component, Default, Clone, Copy)]
pub struct ChannelLatestness(u32);

/// If large octet strings should be broken into smaller packets for transmission. Specific to a channel, may or may not add overhead.
#[derive(Component, Default, Clone, Copy)]
pub struct FragmentedChannel;

/// If messages on this channel should be compressed before transport. This uses the network more efficiently but takes processing on both ends of the connection. Useful with [ChannelFragmentation].
#[derive(Component, Default, Clone, Copy)]
pub enum CompressedChannel {
    /// Compression is slow but the results are smaller.
    High,
    /// Compression is fast but the results may be larger.
    #[default]
    Low,
}