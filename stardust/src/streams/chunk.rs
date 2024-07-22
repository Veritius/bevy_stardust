use bytes::Bytes;

/// A chunk of data received over a stream.
#[derive(Clone)]
pub struct Chunk {
    pub data: Bytes,
}