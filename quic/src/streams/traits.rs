use std::error::Error as StdError;
use bytes::{Buf, Bytes};
use super::StreamId;

/// A type that gives access and control over streams.
pub trait StreamManager {
    /// Handle for the sending side of a stream.
    type Send<'a>: SendStream where Self: 'a;

    /// Handle for the receiving side of a stream.
    type Recv<'a>: RecvStream where Self: 'a;

    /// Opens an outgoing stream.
    fn open_send_stream(&mut self) -> anyhow::Result<StreamId>;

    /// Gets a handle to the sending side of a stream.
    fn get_send_stream(&mut self, id: StreamId) -> Option<Self::Send<'_>>;

    /// Gets a handle to the receiving side of a stream.
    fn get_recv_stream(&mut self, id: StreamId) -> Option<Self::Recv<'_>>;
}

/// A handle to the transmitting side of a QUIC stream.
pub trait SendStream {
    /// An error returned by the underlying QUIC implementation while trying to transmit data.
    type SendError: StdError;

    /// Try to write the contents of `buf` to the stream.
    fn write<B: Buf>(&mut self, buf: &mut B) -> StreamSendOutcome<Self::SendError>;

    /// Finishes the stream, indicating transmission is complete.
    fn finish(&mut self) -> Result<(), Self::SendError>;
    /// Resets the stream, indicating an error and stream close.
    fn reset(&mut self) -> Result<(), Self::SendError>;
}

/// The outcome of trying to write to a QUIC stream.
pub enum StreamSendOutcome<E: StdError> {
    /// Transmitted the full buffer successfully.
    Complete,

    /// Transmitted a part of the chunk.
    /// Contains the amount of bytes transmitted.
    Partial(usize),

    /// The stream is blocked, probably due to congestion control.
    Blocked,

    /// An unexpected error occurred.
    Error(E),
}

/// A handle to the receiving side of a QUIC stream.
pub(crate) trait RecvStream {
    /// An error returned by the underlying QUIC implementation while trying to receive data.
    type RecvError: StdError;

    /// Try to receive chunks from the stream.
    fn recv(&mut self) -> StreamRecvOutcome<Self::RecvError>;

    /// Signals to the remote peer to stop sending, as an error occurred.
    fn stop(&mut self) -> Result<(), Self::RecvError>;
}

/// The outcome of trying to read from a QUIC stream.
pub enum StreamRecvOutcome<E: StdError> {
    /// Received a chunk of information.
    Chunk(Bytes),

    /// No more information to read at the moment, but the stream
    /// isn't finished, and further data may still be received.
    Blocked,

    /// The stream has been stopped. As a result,
    /// no more information can be read from it.
    Stopped,

    /// An unexpected error occurred.
    Error(E),
}