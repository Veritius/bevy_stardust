use quinn_proto::{SendStream, WriteError};

pub(crate) trait StreamWrite {
    fn write(&mut self, data: &[u8]) -> StreamWriteOutcome;
}

pub(crate) enum StreamWriteOutcome {
    Complete,
    Partial(usize),
    Error(WriteError),
}

impl StreamWrite for SendStream<'_> {
    fn write(&mut self, data: &[u8]) -> StreamWriteOutcome {
        match self.write(&data) {
            Ok(written) if written == data.len() => StreamWriteOutcome::Complete,
            Ok(written) => StreamWriteOutcome::Partial(written),
            Err(err) => StreamWriteOutcome::Error(err),
        }
    }
}