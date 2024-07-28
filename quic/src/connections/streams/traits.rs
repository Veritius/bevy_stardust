use bytes::Bytes;
use quinn_proto::StreamId;

pub(crate) trait StreamManager {
    type Outgoing<'a>: SendStream where Self: 'a;

    fn open_send_stream(&mut self) -> anyhow::Result<StreamId>;
    fn get_send_stream(&mut self, id: StreamId) -> Option<Self::Outgoing<'_>>;
}

pub(crate) trait SendStream: StreamTryWrite {
    fn finish_stream(&mut self) -> anyhow::Result<()>;
    fn reset_stream(&mut self) -> anyhow::Result<()>;
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