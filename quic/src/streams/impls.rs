use bytes::{BufMut, Bytes, BytesMut};
use quinn_proto::{Chunks, SendStream, WriteError};
use super::*;

impl WritableStream for BytesMut {
    fn write_to(&mut self, data: Bytes) -> StreamWriteOutcome {
        self.put(data);
        StreamWriteOutcome::Complete
    }

    fn finish(&mut self) -> Result<(), StreamWriteError> {
        unimplemented!()
    }
}

impl WritableStream for SendStream<'_> {
    fn write_to(&mut self, data: Bytes) -> StreamWriteOutcome {
        match self.write_chunks(&mut [data.clone()]) {
            Ok(written) if written.bytes == data.len() => StreamWriteOutcome::Complete,

            Ok(written) => StreamWriteOutcome::Partial(written.bytes),

            Err(WriteError::Blocked) => StreamWriteOutcome::Blocked,

            Err(err) => StreamWriteOutcome::Error(match err {
                WriteError::Stopped(code) => StreamWriteError::Stopped(code.into()),
                WriteError::ClosedStream => StreamWriteError::Closed,
                WriteError::Blocked => unreachable!(),
            }),
        }
    }

    fn finish(&mut self) -> Result<(), StreamWriteError> {
        self.finish().map_err(|e| match e {
            quinn_proto::FinishError::Stopped(code) => StreamWriteError::Stopped(code.into()),
            quinn_proto::FinishError::ClosedStream => StreamWriteError::Closed,
        })
    }
}

impl ReadableStream for Bytes {
    fn read(&mut self) -> StreamReadOutcome {
        if self.len() == 0 { return StreamReadOutcome::Blocked }
        let cloned = self.clone();
        *self = self.slice(self.len()..);
        return StreamReadOutcome::Chunk(cloned)
    }
}

impl ReadableStream for Chunks<'_> {
    fn read(&mut self) -> StreamReadOutcome {
        match self.next(usize::MAX) {
            Ok(Some(chunk)) => StreamReadOutcome::Chunk(chunk.bytes),

            Ok(None) => StreamReadOutcome::Finished,

            Err(error) => match error {
                quinn_proto::ReadError::Blocked => StreamReadOutcome::Blocked,
                quinn_proto::ReadError::Reset(code) => StreamReadOutcome::Error(StreamReadError::Reset(code.into())),
            },
        }
    }
}