use super::serialisation::{ManualBitSerialisation, BitWriter, BitReader, BitstreamError};

pub const CHANNEL_ID_LIMIT: u32 = 2u32.pow(24);

/// A 24-bit channel ID, stored in a u32.
#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChannelId(u32);

impl ChannelId {
    /// Panics with a value greater than 2^24.
    pub(crate) fn new(value: u32) -> Self {
        if value > CHANNEL_ID_LIMIT {
            panic!("Cannot create a ChannelId with a value greater than 2^24");
        }

        Self(value)
    }

    pub(crate) fn from_bytes(bytes: &[u8; 3]) -> Self {
        let mut bytes = [0, bytes[0], bytes[1], bytes[2]];
        Self(u32::from_be_bytes(bytes))
    }
}

impl std::fmt::Debug for ChannelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ChannelId").field(&self.0).finish()
    }
}

impl From<ChannelId> for usize {
    fn from(value: ChannelId) -> Self {
        value.0 as usize
    }
}

impl ManualBitSerialisation for ChannelId {
    fn serialise(&self, writer: &mut impl BitWriter) {
        let bytes = self.0.to_be_bytes();
        for i in 1..3 {
            writer.write_byte(bytes[i]);
        }
    }

    fn deserialise(reader: &mut impl BitReader) -> Result<Self, BitstreamError> {
        let bytes = reader.read_bytes(3)?;
        Ok(Self(u32::from_be_bytes(bytes.try_into().unwrap())))
    }
}

/// Trait for a channel type. Effectively just a marker for `TypeId`s.
pub trait Channel: std::fmt::Debug + Send + Sync + 'static {}

/// Configuration for a network channel.
/// It's recommended to disable features you won't need.
#[derive(Debug, Clone)]
pub struct ChannelConfig {
    /// How many messages are expected to arrive over this channel on the server, per tick. Note this is not per client, this is for all clients combined.
    pub messages_per_tick_server: usize,
    /// How many messages are expected to arrive over this channel on the client, per tick.
    pub messages_per_tick_client: usize,
    pub direction: ChannelDirection,
    pub ordering: ChannelOrdering,
    pub reliability: ChannelReliability,
    pub latestness: ChannelLatestness,
    pub error_checking: ChannelErrorChecking,
    pub fragmentation: ChannelFragmentation,
    pub compression: ChannelCompression,
}

impl ChannelConfig {
    /// Configures an 'essential' channel. Prioritises validity over speed or efficiency.
    /// Use this for messages that *must* be correct.
    pub fn essential(
        messages_per_tick_server: usize,
        messages_per_tick_client: usize,
        direction: ChannelDirection,
    ) -> Self {
        Self {
            messages_per_tick_server,
            messages_per_tick_client,
            direction,
            ordering: ChannelOrdering::Ordered,
            reliability: ChannelReliability::Reliable,
            latestness: ChannelLatestness::Ignore,
            error_checking: ChannelErrorChecking::Enabled,
            fragmentation: ChannelFragmentation::Enabled,
            compression: ChannelCompression::Disabled,
        }
    }

    /// Configures a 'real-time' channel. Prioritises speed over correctness or efficiency.
    /// Messages that arrive late (based on game tick) will be discarded.
    pub fn realtime(
        messages_per_tick_server: usize,
        messages_per_tick_client: usize,
        direction: ChannelDirection,
    ) -> Self {
        Self {
            messages_per_tick_server,
            messages_per_tick_client,
            direction,
            ordering: ChannelOrdering::Unordered,
            reliability: ChannelReliability::Unreliable,
            latestness: ChannelLatestness::Within(3),
            error_checking: ChannelErrorChecking::Disabled,
            fragmentation: ChannelFragmentation::Disabled,
            compression: ChannelCompression::Disabled,
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
/// Messages sent in the wrong direction will be discarded.
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

/// Whether or not to discard messages if they are X ticks late.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChannelLatestness {
    Ignore,
    Within(u32),
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