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
        return Some(SendStream { inner: self.0, id });;
    }
    
    fn get_recv_stream(&mut self, id: StreamId) -> Option<Self::Recv<'_>> {
        return Some(RecvStream { inner: self.0, id });
    }
}

pub struct RecvStream<'a> {
    inner: &'a mut QuicheConnection,
    id: StreamId,
}

impl<'a> crate::connection::RecvStream for RecvStream<'a> {
    type RecvError = quiche::Error;

    fn recv(&mut self) -> StreamRecvOutcome<Self::RecvError> {
        todo!()
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

    fn send<B: bytes::Buf>(&mut self, buf: &mut B) -> StreamSendOutcome<Self::SendError> {
        let total = buf.remaining();
        let mut written = 0;

        while buf.remaining() > 0 && written <= total {
            match self.inner.connection.stream_send(
                self.id.inner(),
                buf.chunk(),
                false,
            ) {
                Ok(amt) => {
                    written += amt;
                    buf.advance(amt);
                    continue;
                }

                Err(error) => return match error {
                    // Stream has no capacity (from stream_send docs)
                    quiche::Error::Done => StreamSendOutcome::Blocked,

                    // Stream was stopped/reset
                    quiche::Error::StreamStopped(_) => StreamSendOutcome::Stopped,
                    quiche::Error::StreamReset(_) => StreamSendOutcome::Stopped,

                    // The rest of the cases are actual errors
                    error => StreamSendOutcome::Error(error),
                },
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