use std::{ops::Deref, str::{from_utf8, Utf8Error}};
use bytes::Bytes;
use super::ChannelId;

/// Individual, cheaply clonable, contiguous octet (byte) strings.
/// 
/// An individual message has the following guarantees:
/// - Complete: messages are never received piecemeal.
/// - Correct: a received message is exactly what was sent.
/// 
/// These guarantees **are not** enforced by this type.
/// Instead, they are enforced by the transport layer handling I/O.
/// These guarantees are also not to be used for unsafe code.
/// All messages should be considered untrusted input, and
/// defensive programming should be employed when using them.
/// 
/// A `Message` is a wrapper around a [`Bytes`], with an intentionally simplified API.
/// Functions like [`Bytes::slice`] are not present, as a `Message` is complete.
/// If you want to use slicing operations, convert this into a `Bytes` with [`Into<Bytes>`].
/// [`From<Bytes>`] exists for use by transport layers. A slice of a `Message` is considered
/// a violation of the 'completeness' guarantee.
#[derive(Clone)]
#[repr(transparent)]
pub struct Message(Bytes);

impl Message {
    /// Returns the length of the message, in bytes.
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the length of the message is `0`.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Attempts to read the message as a string slice.
    /// 
    /// This fails if the message is not valid UTF-8.
    /// See the [`from_utf8`] docs for more information.
    #[inline]
    pub fn as_str(&self) -> Result<&str, Utf8Error> {
        from_utf8(&self.0)
    }

    /// Create a [`Message`] from a [`Bytes`].
    /// 
    /// This is different from the [`From<Bytes>`] implementation,
    /// as it can be used in const contexts.
    #[inline]
    pub const fn from_bytes(bytes: Bytes) -> Self {
        Self(bytes)
    }

}

impl From<Bytes> for Message {
    #[inline]
    fn from(bytes: Bytes) -> Self {
        Self::from_bytes(bytes)
    }
}

impl From<Message> for Bytes {
    #[inline]
    fn from(message: Message) -> Self {
        message.0
    }
}

impl Deref for Message {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0[..]
    }
}

impl std::fmt::Debug for Message {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// A [`Message`] with an associated [`ChannelId`].
#[derive(Clone)]
pub struct ChannelMessage {
    /// The channel's identifier.
    pub channel: ChannelId,

    /// The contents of the message.
    pub payload: Message,
}

impl From<(ChannelId, Message)> for ChannelMessage {
    fn from(value: (ChannelId, Message)) -> Self {
        Self {
            channel: value.0,
            payload: value.1,
        }
    }
}

impl From<(ChannelId, Bytes)> for ChannelMessage {
    fn from(value: (ChannelId, Bytes)) -> Self {
        Self {
            channel: value.0,
            payload: Message::from_bytes(value.1),
        }
    }
}