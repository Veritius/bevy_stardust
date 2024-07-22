use std::ops::Deref;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct StreamId(u64);

impl StreamId {
    pub fn new(inner: u64) -> Result<Self, ()> {

        return Ok(Self(inner));
    }

    pub fn inner(&self) -> u64 {
        self.0
    }
}

impl Deref for StreamId {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<StreamId> for u64 {
    fn from(value: StreamId) -> Self {
        value.0
    }
}