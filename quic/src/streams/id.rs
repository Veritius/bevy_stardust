#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct StreamId(StreamIdInner);

#[cfg(feature="quiche")]
type StreamIdInner = u64;

impl StreamId {
    pub fn inner(&self) -> StreamIdInner {
        self.0
    }
}