use std::error::Error as StdError;
use bytes::{Buf, Bytes};

/// A type that gives access and control over datagrams.
pub trait DatagramManager {
    /// An error returned by the underlying QUIC implementation while trying to transmit data.
    type SendError: StdError + Send + Sync + 'static;

    /// An error returned by the underlying QUIC implementation while trying to receive data.
    type RecvError: StdError + Send + Sync + 'static;

    /// The maximum size of datagrams that can be sent.
    fn max_size(&self) -> usize;

    /// Try to send `buf` as a datagram.
    /// The entire datagram must be sent, or not at all.
    fn send<B: Buf>(&mut self, buf: &mut B) -> Result<(), Self::SendError>;

    /// Try to receive a single datagram.
    /// The entire datagram must be received, or not at all.
    fn recv(&mut self) -> Result<Bytes, Self::RecvError>;
}