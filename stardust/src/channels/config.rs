//! Channel configuration.
//! 
//! All settings are not definitive, but hints to transport layers as how to treat channels.

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

/// If a packet is missed, it will be resent. This can take a (relatively) long time.
/// If used with [ordering](ChannelOrdering::Ordered) this can cause [head-of-line blocking].
/// 
/// [head-of-line blocking]: https://en.wikipedia.org/wiki/Head-of-line_blocking
#[derive(Debug, Hash)]
pub enum ChannelReliability {
    /// Messages will not be resent if missed.
    Unreliable,
    /// Messages will be resent if missed.
    Reliable,
}

/// Ensure that systems read octet strings in the exact order they were sent over the wire.
/// Internet infrastructure doesn't guarantee the order of arrival, so it must be dealt with by the software.
#[derive(Debug, Hash)]
pub enum ChannelOrdering {
    /// Messages will be read in the order they arrive.
    Unordered,
    /// Messages will be read in the order they were sent.
    Ordered,
}

/// If a message is too large to send in a single packet, it'll instead be sent in multiple pieces, and recombined later.
/// It's highly recommended to use [reliability](ChannelReliability::Reliable), since most transport layers will discard the entire thing if one packet is missed.
#[derive(Debug, Hash)]
pub enum ChannelFragmentation {
    /// Disable fragmentation.
    Disabled,
    /// Enable fragmentation.
    Enabled,
}

/// Compresses octet strings on channels, reducing size over the wire at the cost of extra processing on both ends.
/// 
/// Most octet strings don't need compression. In general, you'll only need it if you're sending more than a few kilobytes.
/// It's highly recommended to also use [fragmentation](ChannelFragmentation::Enabled) along with this, since any messages that are compressed
/// are likely to still be too big to be sent in a single packet.
#[derive(Debug, Hash)]
pub enum ChannelCompression {
    /// Don't compress messages.
    Disabled,
    /// Compress messages, but don't sacrifice speed.
    Fast,
    /// Compress messages as much as possible.
    High,
}

/// Tries to ensure that the message is received exactly as it was sent.
/// This does not protect against a [MITM attack] by itself, use signing or encryption for that (if your transport layer supports it).
/// 
/// [MITM attack]: https://en.wikipedia.org/wiki/Man-in-the-middle_attack
#[derive(Debug, Hash)]
pub enum MessageValidation {
    /// The integrity of messages will not be checked.
    Disabled,
    /// The integrity of messages will be checked.
    Enabled,
}