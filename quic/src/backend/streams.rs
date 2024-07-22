use bytes::{Buf, Bytes};

/// A type that gives access and control over streams.
pub trait StreamManager {
    /// Handle for the sending side of a stream.
    type Send<'s>: SendStream where Self: 's;

    /// Handle for the receiving side of a stream.
    type Recv<'s>: RecvStream where Self: 's;

    /// Opens an outgoing stream.
    fn open_send_stream(&mut self) -> anyhow::Result<StreamId>;

    /// Gets a handle to the sending side of a stream.
    fn get_send_stream(&mut self, id: StreamId) -> Option<Self::Send<'_>>;

    /// Gets a handle to the receiving side of a stream.
    fn get_recv_stream(&mut self, id: StreamId) -> Option<Self::Recv<'_>>;

    /// Return the next send stream that data can be transmitted on.
    fn next_available_send(&mut self) -> Option<Self::Send<'_>>;

    /// Return the next recv stream that data can be transmitted on.
    fn next_available_recv(&mut self) -> Option<Self::Recv<'_>>;
}

/// A handle to the transmitting side of a QUIC stream.
pub trait SendStream {
    /// An error returned by the underlying QUIC implementation while trying to transmit data.
    type SendError: Into<anyhow::Error>;

    /// Returns the stream's unique ID.
    fn id(&self) -> StreamId;

    /// Set the priority of a stream.
    fn priority(&mut self, priority: u32) -> Result<(), Self::SendError>;

    /// Try to write the contents of `buf` to the stream.
    fn send<B: Buf>(&mut self, buf: &mut B) -> StreamSendOutcome<Self::SendError>;

    /// Finishes the stream, indicating transmission is complete.
    fn finish(&mut self) -> Result<(), Self::SendError>;
    /// Resets the stream, indicating an error and stream close.
    fn reset(&mut self) -> Result<(), Self::SendError>;
}

/// The outcome of trying to write to a QUIC stream.
pub enum StreamSendOutcome<E>
where
    E: Into<anyhow::Error>,
{
    /// Transmitted the full buffer successfully.
    Complete,

    /// Transmitted a part of the chunk.
    /// Contains the amount of bytes transmitted.
    Partial(usize),

    /// The stream is blocked, probably due to congestion control.
    /// Attempting to send data in the future may work.
    Blocked,

    /// The stream is stopped, either due to a finish, reset, or stop.
    /// Once this is sent, no further data can be transmitted.
    Stopped,

    /// An unexpected error occurred.
    Error(E),
}

/// A handle to the receiving side of a QUIC stream.
pub trait RecvStream {
    /// An error returned by the underlying QUIC implementation while trying to receive data.
    type RecvError: Into<anyhow::Error>;

    /// Returns the stream's unique ID.
    fn id(&self) -> StreamId;

    /// Try to receive chunks from the stream.
    fn recv(&mut self) -> StreamRecvOutcome<Self::RecvError>;

    /// Signals to the remote peer to stop sending, as an error occurred.
    fn stop(&mut self) -> Result<(), Self::RecvError>;
}

/// The outcome of trying to read from a QUIC stream.
pub enum StreamRecvOutcome<E>
where
    E: Into<anyhow::Error>,
{
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StreamId(u64);

impl StreamId {
    pub const MAX: u64 = 2u64.pow(62) - 1;

    pub fn new(inner: u64) -> Result<Self, ()> {
        if inner > Self::MAX { return Err(()); }
        return Ok(Self(inner));
    }

    pub fn inner(self) -> u64 {
        self.0
    }
}