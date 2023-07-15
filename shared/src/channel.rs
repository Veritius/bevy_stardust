pub struct ChannelConfig {
    /// Max storage space for incoming messages, in kilobytes.
    pub memory_size: u32,
    /// The direction messages can flow in.
    pub direction: ChannelDirection,
    /// The reliability of messages (resending, ordering)
    pub reliability: ChannelReliability,
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
pub enum ChannelReliability {
    /// Messages will arrive in order.
    OrderedReliable,
    /// Messages will arrive.
    UnorderedReliable,
    /// Messages may arrive.
    Unreliable
}