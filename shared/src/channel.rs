/// Trait for a channel type. Effectively just a marker for `TypeId`s.
pub trait Channel: Send + Sync + 'static {}

/// Configuration for a network channel.
#[derive(Debug, Clone)]
pub struct ChannelConfig {
    pub direction: ChannelDirection,
    pub ordering: ChannelOrdering,
    pub mode: ChannelReliability,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChannelDirection {
    /// Messages can be sent from and received by both the client and the server.
    Bidirectional,
    /// Messages can be sent from the server and received by the client.
    ServerToClient,
    /// Messages can be sent from the client and received by the server.
    ClientToServer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChannelOrdering {
    Ordered,
    Unordered,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChannelReliability {
    /// Messages will arrive eventually.
    Reliable,
    /// Messages may or may not arrive.
    Unreliable,
}