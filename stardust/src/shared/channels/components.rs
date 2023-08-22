use std::any::TypeId;
use bevy::prelude::*;
use super::id::ChannelId;

/// An entity representing a channel.
#[derive(Component)]
pub struct ChannelData {
    pub(crate) type_id: TypeId,
    pub(crate) type_path: &'static str,
    pub(crate) channel_id: ChannelId,
}

impl ChannelData {
    /// Returns the associated `TypeId` used to access this channel.
    pub fn type_id(&self) -> TypeId {
        self.type_id
    }

    /// Returns the [TypePath](https://docs.rs/bevy_reflect/0.11.0/bevy_reflect/trait.TypePath.html) fully qualified path.
    pub fn type_path(&self) -> &'static str {
        self.type_path
    }

    /// Returns the associated `ChannelId` used for network transport.
    pub fn channel_id(&self) -> ChannelId {
        self.channel_id
    }
}

/// Marks this channel as directional.
#[derive(Debug, Component, Hash, Clone, Copy, PartialEq, Eq)]
pub enum DirectionalChannel {
    /// Only the server can send messages on this channel.
    ServerToClient,
    /// Only a client can send a message on this channel.
    ClientToServer,
}

/// Marks this channel as ordered - messages sent in this channel will arrive in the exact order they are sent. Messages may not arrive, use [ReliableChannel] to ensure they do.
#[derive(Component, Default, Hash, Clone, Copy)]
pub struct OrderedChannel;

/// Marks this channel as reliable - messages sent in this channel are guaranteed to arrive eventually.
#[derive(Component, Default, Hash, Clone, Copy)]
pub struct ReliableChannel;

/// Discards packets in this channel that are older than a certain amount of ticks.
#[derive(Component, Default, Hash, Clone, Copy)]
pub struct ChannelLatestness(u32);

/// If large octet strings should be broken into smaller packets for transmission. Specific to a channel, may or may not add overhead.
#[derive(Component, Default, Hash, Clone, Copy)]
pub struct FragmentedChannel;

/// If messages on this channel should be compressed before transport. This uses the network more efficiently but takes processing on both ends of the connection. Useful with [ChannelFragmentation].
#[derive(Component, Default, Hash, Clone, Copy, PartialEq, Eq)]
pub enum CompressedChannel {
    /// Compression is slow but the results are smaller.
    High,
    /// Compression is fast but the results may be larger.
    #[default]
    Low,
}