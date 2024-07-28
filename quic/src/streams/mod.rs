mod incoming;
mod outgoing;

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