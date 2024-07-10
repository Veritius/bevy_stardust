use bytes::Bytes;
use super::StreamId;

pub(crate) trait StreamManager {
    type SendStream: StreamTryWrite;

    fn get_send_stream(&mut self, id: StreamId) -> Option<Self::SendStream>;
}

pub(crate) trait StreamTryWrite {
    fn try_write_stream(&mut self, chunk: Bytes) -> StreamTryWriteOutcome;
}

pub(crate) enum StreamTryWriteOutcome {
    Complete,
    Partial(usize),
    Blocked,
    Error(anyhow::Error),
}