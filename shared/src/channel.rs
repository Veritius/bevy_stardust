/// Trait for a channel type. Effectively just a marker for `TypeId`s.
pub trait Channel: Send + Sync + 'static {}

/// Configuration for a network channel.
/// It's recommended to disable features you won't need.
#[derive(Debug, Clone)]
pub struct ChannelConfig {
    pub direction: ChannelDirection,
    pub ordering: ChannelOrdering,
    pub reliability: ChannelReliability,
    pub error_checking: ChannelErrorChecking,
    pub fragmentation: ChannelFragmentation,
    pub compression: ChannelCompression,
    pub latest_only: bool,
}

impl ChannelConfig {
    /// Configures a 'real-time' channel. Prioritises speed over correctness or efficiency.
    /// Messages that arrive late (based on game tick) will be discarded.
    fn realtime(direction: ChannelDirection) -> Self {
        Self {
            direction,
            ordering: ChannelOrdering::Unordered,
            reliability: ChannelReliability::Unreliable,
            error_checking: ChannelErrorChecking::Disabled,
            fragmentation: ChannelFragmentation::Disabled,
            compression: ChannelCompression::Disabled,
            latest_only: true,
        }
    }
}

/// Defines the direction messages in this channel can flow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChannelDirection {
    /// Messages can be sent from and received by both the client and the server.
    Bidirectional,
    /// Messages can be sent from the server and received by the client.
    ServerToClient,
    /// Messages can be sent from the client and received by the server.
    ClientToServer,
}

/// Whether or not messages should be read in the order they arrive in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChannelOrdering {
    Ordered,
    Unordered,
}

/// Whether or not messages are guaranteed to arrive. Reliable messages may be slower to arrive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChannelReliability {
    /// Messages will arrive eventually.
    Reliable,
    /// Messages may or may not arrive.
    Unreliable,
}

/// Whether or not messages in this channel will be error checked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChannelErrorChecking {
    Enabled,
    Disabled,
}

/// Whether or not messages too large to fit in a single packet will be split into multiple.
/// If an oversized message is sent in a non-fragmented channel, Stardust will panic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChannelFragmentation {
    Enabled,
    Disabled,
}

/// Whether or not the payload data in the message should be compressed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChannelCompression {
    Enabled,
    Disabled,
}