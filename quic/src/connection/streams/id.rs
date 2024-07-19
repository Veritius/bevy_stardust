#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct StreamId(StreamIdInner);

#[cfg(feature="quiche")]
type StreamIdInner = u64;

impl StreamId {
    pub fn new(inner: StreamIdInner) -> Self {
        Self(inner)
    }

    pub fn inner(&self) -> StreamIdInner {
        self.0
    }
}