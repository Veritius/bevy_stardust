use bytes::Bytes;

pub(crate) trait StreamTryWrite {
    fn try_write(data: &[Bytes]) -> StreamTryWriteOutcome;
}

pub(crate) enum StreamTryWriteOutcome {
    Complete,
    Partial(usize),
    Blocked,
    Error(anyhow::Error),
}