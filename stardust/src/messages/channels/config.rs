use bevy::reflect::Reflect;

/// Configuration for a channel.
#[derive(Debug, Clone, Hash, Reflect)]
#[reflect(Debug, Hash)]
pub struct ChannelConfiguration {
    /// Guarantees that the transport layer must make
    /// for messages sent on this channel. See the
    /// documentation of [`ChannelConsistency`].
    pub consistency: ChannelConsistency,

    /// The priority of messages on this channel.
    /// Transport values will send messages on channels with higher `priority` values first.
    pub priority: u32,
}

/// Reliability and ordering guarantees.
/// This is enforced by the transport layer handling the client.
/// 
/// # Why?
/// ## Reliability
/// <https://en.wikipedia.org/wiki/Reliability_(computer_networking)>
/// 
/// The Internet makes no guarantees about your message being received.
/// This is a challenge if your application is expecting something, and it's lost.
/// Reliability guarantees that individual messages on this channel are received
/// eventually, through whatever means are available to the transport layer.
/// This almost always incurs some overhead, and may be undesirable for
/// certain kinds of transmission, especially for real-time data.
/// 
/// ## Ordering
/// The Internet makes no guarantees about the order packets are received in.
/// This means that if you're trying to send chunks of an image, you may
/// receive packets in the wrong order to the one they were sent in, and end
/// up with a very muddled up image.
/// 
/// By enabling ordering for a channel, transport layers will ensure
/// that messages in the channel will be received in a specified order,
/// relative to the order they were sent in. Messages are only ordered
/// against other messages in the same channel.
/// 
/// Sequencing is related to ordering, but discards older messages when
/// an out-of-order transmission occurs. If the messages `[1,2,3,4,5]` is
/// received in order, the application sees `[1,2,3,4,5]`. However, if the
/// messages are received in the order `[1,3,2,5,4]`, the application will
/// only see the messages `[1,3,5]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
#[reflect(Debug, PartialEq, Hash)]
#[non_exhaustive]
pub enum ChannelConsistency {
    /// Messages lost in transport will not be resent.
    /// They are added to the queue in the order they're received,
    /// which may be different to the order they were sent in.
    /// 
    /// Useful for messages that can occasionally be lost,
    /// and aren't needed to be read in a specific order,
    /// such as spawning particle effects.
    UnreliableUnordered,

    /// Messages lost in transport will not be resent.
    /// If messages are not received in order, only the most
    /// recent messages will be stored, discarding old messages.
    /// 
    /// Useful for messages that are used to update something
    /// each tick, where only the most recent values matter,
    /// such as player position in a shooter.
    UnreliableSequenced,

    /// Messages lost in transport will be resent.
    /// They are added to the queue in the order they're received,
    /// which may be different to the order they were sent in.
    /// 
    /// Useful for messages that must be received,
    /// but don't have any ordering requirements,
    /// such as inventory updates in a survival game.
    ReliableUnordered,

    /// Messages lost in transport will be resent.
    /// They are added to the queue in the order they were sent,
    /// which may introduce a delay in the case of a resend.
    /// 
    /// Useful for messages that must be received,
    /// and must be received in a certain order,
    /// such as chat messages in a multiplayer game.
    ReliableOrdered,
}

impl ChannelConsistency {
    /// Returns `true` if messages in this channel must be sent reliably.
    pub fn is_reliable(&self) -> bool {
        match self {
            ChannelConsistency::UnreliableUnordered => false,
            ChannelConsistency::UnreliableSequenced => false,
            ChannelConsistency::ReliableUnordered   => true,
            ChannelConsistency::ReliableOrdered     => true,
        }
    }

    /// Returns `true` if messages in this channel have any ordering constraints applied.
    pub fn is_ordered(&self) -> bool {
        match self {
            ChannelConsistency::UnreliableUnordered => false,
            ChannelConsistency::UnreliableSequenced => true,
            ChannelConsistency::ReliableUnordered   => false,
            ChannelConsistency::ReliableOrdered     => true,
        }
    }
}