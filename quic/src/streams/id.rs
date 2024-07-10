#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct StreamId(StreamIdInner);

#[cfg(feature="quiche")]
type StreamIdInner = u64;