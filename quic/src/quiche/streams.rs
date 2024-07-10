use quiche::Shutdown;

use crate::streams::*;
use super::QuicheConnection;

impl StreamManager for QuicheConnection {
    type Outgoing<'a> = QuicheSendStream<'a>;
    
    fn open_send_stream(&mut self) -> anyhow::Result<StreamId> {
        let v = self.out_sid_idx;
        self.out_sid_idx += 1;
        return Ok(StreamId::new(v));
    }
    
    fn get_send_stream(&mut self, id: StreamId) -> Option<Self::Outgoing<'_>> {
        Some(QuicheSendStream { id, connection: self })
    }
}

pub struct QuicheSendStream<'a> {
    id: StreamId,
    connection: &'a mut super::QuicheConnection,
}

impl<'a> SendStream for QuicheSendStream<'a> {
    fn finish_stream(&mut self) -> anyhow::Result<()> {
        self.connection.inner.stream_send(self.id.inner(), &[], true)?;
        return Ok(());
    }

    fn reset_stream(&mut self) -> anyhow::Result<()> {
        self.connection.inner.stream_shutdown(self.id.inner(), Shutdown::Write, 0)?;
        return Ok(());
    }
}

impl<'a> StreamTryWrite for QuicheSendStream<'a> {
    fn try_write_stream(&mut self, chunk: bytes::Bytes) -> StreamTryWriteOutcome {
        match self.connection.inner.stream_send(self.id.inner(), &chunk[..], false) {
            Ok(written) => match written == chunk.len() {
                true => StreamTryWriteOutcome::Complete,
                false => StreamTryWriteOutcome::Partial(written),
            },

            Err(quiche::Error::Done) => StreamTryWriteOutcome::Blocked,
            Err(err) => StreamTryWriteOutcome::Error(err.into()),
        }
    }
}