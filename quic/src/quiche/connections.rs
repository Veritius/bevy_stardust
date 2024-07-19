use crate::connection::{ConnectionState, DatagramManager, StreamId, StreamManager};

pub struct QuicheConnection {
    connection: quiche::Connection,
}

impl ConnectionState for QuicheConnection {
    type Backend = super::Quiche;

    type Datagrams<'a> where Self: 'a = Self;
    
    type Streams<'a> where Self: 'a = Self;

    fn is_closed(&self) -> bool {
        todo!()
    }
    
    fn datagrams(&mut self) -> Self::Datagrams<'_> {
        todo!()
    }
    
    fn streams(&mut self) -> Self::Streams<'_> {
        todo!()
    }
}

impl DatagramManager for QuicheConnection {
    type RecvError = quiche::Error;
    type SendError = quiche::Error;

    fn max_size(&self) -> usize {
        self.connection.dgram_max_writable_len().unwrap() // TODO: Handle None case
    }

    fn recv(&mut self) -> Result<bytes::Bytes, Self::RecvError> {
        self.connection.dgram_recv_vec().map(|v| v.into())
    }

    fn send<B: bytes::Buf>(&mut self, buf: &mut B) -> Result<(), Self::SendError> {
        self.connection.dgram_send(&buf.copy_to_bytes(buf.remaining()))
    }
}

impl StreamManager for QuicheConnection {
    type Recv<'a> = RecvStream<'a>;
    type Send<'a> = SendStream<'a>;

    fn open_send_stream(&mut self) -> anyhow::Result<StreamId> {
        todo!()
    }
    
    fn get_send_stream(&mut self, id: StreamId) -> Option<Self::Send<'_>> {
        todo!()
    }
    
    fn get_recv_stream(&mut self, id: StreamId) -> Option<Self::Recv<'_>> {
        todo!()
    }
}

pub struct RecvStream<'a> {
    connection: &'a mut QuicheConnection,
    id: StreamId
}

impl<'a> crate::connection::RecvStream for RecvStream<'a> {
    type RecvError = quiche::Error;

    fn recv(&mut self) -> crate::connection::StreamRecvOutcome<Self::RecvError> {
        todo!()
    }

    fn stop(&mut self) -> Result<(), Self::RecvError> {
        todo!()
    }
}

pub struct SendStream<'a> {
    connection: &'a mut QuicheConnection,
    id: StreamId
}

impl<'a> crate::connection::SendStream for SendStream<'a> {
    type SendError = quiche::Error;

    fn send<B: bytes::Buf>(&mut self, buf: &mut B) -> crate::connection::StreamSendOutcome<Self::SendError> {
        todo!()
    }

    fn finish(&mut self) -> Result<(), Self::SendError> {
        todo!()
    }

    fn reset(&mut self) -> Result<(), Self::SendError> {
        todo!()
    }
}