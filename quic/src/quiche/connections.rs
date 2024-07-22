use bytes::Bytes;

use crate::connection::{ConnectionState, DatagramManager, StreamId, StreamManager, StreamRecvOutcome, StreamSendOutcome};

pub struct QuicheConnection {
    connection: quiche::Connection,
}

impl ConnectionState for QuicheConnection {
    type Backend = super::Quiche;

    type Datagrams<'a> = QuicheDatagrams<'a>;
    type Streams<'a> = QuicheStreams<'a>;

    fn is_closed(&self) -> bool {
        self.connection.is_closed()
    }

    fn datagrams(&mut self) -> Self::Datagrams<'_> {
        QuicheDatagrams(self)
    }

    fn streams(&mut self) -> Self::Streams<'_> {
        QuicheStreams(self)
    }
}

pub struct QuicheDatagrams<'a>(&'a mut QuicheConnection);

impl<'a> DatagramManager for QuicheDatagrams<'a> {
    type RecvError = quiche::Error;
    type SendError = quiche::Error;

    fn max_size(&self) -> usize {
        self.0.connection.dgram_max_writable_len().unwrap() // TODO: Handle None case
    }

    fn recv(&mut self) -> Result<bytes::Bytes, Self::RecvError> {
        self.0.connection.dgram_recv_vec().map(|v| v.into())
    }

    fn send<B: bytes::Buf>(&mut self, buf: &mut B) -> Result<(), Self::SendError> {
        self.0.connection.dgram_send(&buf.copy_to_bytes(buf.remaining()))
    }
}

pub struct QuicheStreams<'a>(&'a mut QuicheConnection);

impl<'a> StreamManager for QuicheStreams<'a> {
    type Recv<'s> = RecvStream<'s> where Self: 's;
    type Send<'s> = SendStream<'s> where Self: 's;

    fn open_send_stream(&mut self) -> anyhow::Result<StreamId> {
        todo!()
    }
    
    fn get_send_stream(&mut self, id: StreamId) -> Option<Self::Send<'_>> {
        return Some(SendStream { inner: self.0, id });
    }
    
    fn get_recv_stream(&mut self, id: StreamId) -> Option<Self::Recv<'_>> {
        if self.0.connection.stream_finished(id.inner()) { return None; }
        return Some(RecvStream { inner: self.0, id });
    }

    fn next_available_send(&mut self) -> Option<Self::Send<'_>> {
        let id = self.0.connection.stream_writable_next()?;
        return Some(SendStream { inner: self.0, id: StreamId::new(id).unwrap() });
    }

    fn next_available_recv(&mut self) -> Option<Self::Recv<'_>> {
        let id = self.0.connection.stream_readable_next()?;
        return Some(RecvStream { inner: self.0, id: StreamId::new(id).unwrap() });
    }
}

pub struct RecvStream<'a> {
    inner: &'a mut QuicheConnection,
    id: StreamId,
}

const RECV_SCRATCH_ALLOC_SIZE: usize = 1024;

impl<'a> crate::connection::RecvStream for RecvStream<'a> {
    type RecvError = quiche::Error;

    fn id(&self) -> StreamId {
        self.id
    }

    fn recv(&mut self) -> StreamRecvOutcome<Self::RecvError> {
        // Check if the stream is readable in the first place
        if !self.inner.connection.stream_readable(self.id.inner()) {
            return StreamRecvOutcome::Blocked;
        }

        // Scratch vector for stream_recv to work with
        let mut scratch = vec![0u8; RECV_SCRATCH_ALLOC_SIZE];

        match self.inner.connection.stream_recv(self.id.inner(), &mut scratch[..]) {
            Ok((len, _fin)) => {
                // TODO: Avoid allocating twice, maybe reuse the vec's allocation?
                // SAFETY: We only take up to the written part of memory
                let chunk = Bytes::copy_from_slice(&scratch[..len]);
                return StreamRecvOutcome::Chunk(chunk);
            },

            Err(quiche::Error::Done) => StreamRecvOutcome::Blocked,
            Err(error) => StreamRecvOutcome::Error(error),
        }
    }

    fn stop(&mut self) -> Result<(), Self::RecvError> {
        todo!()
    }
}

pub struct SendStream<'a> {
    inner: &'a mut QuicheConnection,
    id: StreamId,
}

impl<'a> crate::connection::SendStream for SendStream<'a> {
    type SendError = quiche::Error;

    fn id(&self) -> StreamId {
        self.id
    }

    fn send<B: bytes::Buf>(&mut self, buf: &mut B) -> StreamSendOutcome<Self::SendError> {
        let total = buf.remaining();
        let mut written = 0;

        while buf.remaining() > 0 && written <= total {
            match self.inner.connection.stream_send(
                self.id.inner(),
                buf.chunk(),
                false,
            ) {
                // Successfully sent data
                Ok(amt) => {
                    written += amt;
                    buf.advance(amt);
                    continue;
                }

                // Stream has no capacity (from stream_send docs)
                Err(quiche::Error::Done) => return StreamSendOutcome::Blocked,

                // Stream was stopped/reset
                Err(quiche::Error::StreamStopped(_)) => return StreamSendOutcome::Stopped,
                Err(quiche::Error::StreamReset(_)) => return StreamSendOutcome::Stopped,

                // The rest of the cases are actual errors
                Err(error) => return StreamSendOutcome::Error(error),
            }
        }

        match total == written {
            true => StreamSendOutcome::Complete,
            false => StreamSendOutcome::Partial(written),
        }
    }

    fn finish(&mut self) -> Result<(), Self::SendError> {
        todo!()
    }

    fn reset(&mut self) -> Result<(), Self::SendError> {
        todo!()
    }
}