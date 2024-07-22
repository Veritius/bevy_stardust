use super::{DatagramManager, QuicBackend, StreamManager};

pub use crate::endpoint::ConnectionId;

/// Connection state for a connection managed by a [`QuicBackend`] implementor.
pub trait ConnectionState
where
    Self: Send + Sync,
{
    /// The [`QuicBackend`] implementation that manages this connection.
    type Backend: QuicBackend;

    /// A handle returned by the [`datagrams`](ConnectionState::datagrams) method.
    type Datagrams<'a>: DatagramManager where Self: 'a;

    /// A handle returned by the [`streams`](ConnectionState::streams) method.
    type Streams<'a>: StreamManager where Self: 'a;

    /// Returns `true` if the connection is fully closed and drained,
    /// and that dropping it is guaranteed to not cause data loss.
    fn is_closed(&self) -> bool;

    /// Get a handle to a `DatagramManager` for this connection.
    fn datagrams(&mut self) -> Self::Datagrams<'_>;

    /// Get a handle to a `StreamManager` for this connection.
    fn streams(&mut self) -> Self::Streams<'_>;
}