mod events;
mod framing;
mod header;
mod incoming;

use crate::Connection;
use bytes::Bytes;

pub(crate) use incoming::IncomingStream;

pub use events::StreamEvent;

impl Connection {
    /// Call when a new incoming stream is opened.
    pub fn stream_open(&mut self, stream: StreamId) {
        todo!()
    }

    /// Call when a chunk of data is received on a stream.
    pub fn stream_recv(&mut self, stream: StreamId, chunk: Bytes) {
        todo!()
    }

    /// Call when a stream is stopped.
    pub fn stream_stopped(&mut self, stream: StreamId) {
        todo!()
    }

    /// Call when a stream is reset.
    pub fn stream_reset(&mut self, stream: StreamId) {
        todo!()
    }

    /// Call when a stream is finish.
    pub fn stream_finished(&mut self, stream: StreamId) {
        todo!()
    }
}

/// A stream identifier value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StreamId(u64);

impl StreamId {
    const MAX: u64 = 2u64.pow(62) - 1;

    /// Creates a new [`StreamId`], checking if it's valid.
    pub fn new(value: u64) -> Result<Self, ()> {
        if value > Self::MAX { return Err(()) }
        return Ok(Self(value));
    }

    /// Creates a new [`StreamId`] without checking if it's valid.
    /// 
    /// # SAFETY
    /// You must not exceed the limit for stream ids defined in RFC 9000, which is `(2^62)-1`
    #[inline]
    pub const unsafe fn new_unchecked(value: u64) -> Self {
        Self(value)
    }

    /// Returns the internal value, which is guaranteed to be below `(2^62)-1`.
    #[inline]
    pub const fn inner(self) -> u64 {
        self.0
    }
}

impl TryFrom<u64> for StreamId {
    type Error = ();

    #[inline]
    fn try_from(value: u64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<StreamId> for u64 {
    #[inline]
    fn from(value: StreamId) -> Self {
        value.inner()
    }
}