use std::{ops::Deref, str::{from_utf8, Utf8Error}};
use bytes::Bytes;
use crate::channels::ChannelId;

/// An individual, whole message. The most basic communication primitive.
/// 
/// Messages are cheaply clonable and contiguous, being a `#[repr(transparent)]` wrapper around a [`Bytes`].
/// 
/// ## Constraints
/// A `Message` is **unaltered** - it is exactly the same series of bytes as what was sent by the peer.
/// All outside data is untrusted, and you should employ defensive programming when handling user data.
/// It's recommended to use the `untrusted` crate or the `octs` crate, but not `bytes`,
/// since `bytes` methods panic rather than returning an error.
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

    /// Returns the message as a slice of bytes.
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        &self.0
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

    /// Create a [`Message`] from a static slice of bytes.
    #[inline]
    pub const fn from_static(slc: &'static [u8]) -> Self {
        Self::from_bytes(Bytes::from_static(slc))
    }

    /// Create a [`Message`] from a static string slice.
    #[inline]
    pub const fn from_static_str(str: &'static str) -> Self {
        Self::from_static(str.as_bytes())
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

impl AsRef<[u8]> for Message {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.0[..]
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
    pub message: Message,
}

impl From<(ChannelId, Message)> for ChannelMessage {
    fn from(value: (ChannelId, Message)) -> Self {
        Self {
            channel: value.0,
            message: value.1,
        }
    }
}

impl From<(ChannelId, Bytes)> for ChannelMessage {
    fn from(value: (ChannelId, Bytes)) -> Self {
        Self {
            channel: value.0,
            message: Message::from_bytes(value.1),
        }
    }
}

impl AsRef<Message> for ChannelMessage {
    #[inline]
    fn as_ref(&self) -> &Message {
        &self.message
    }
}

impl AsRef<[u8]> for ChannelMessage {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.message.as_ref()
    }
}