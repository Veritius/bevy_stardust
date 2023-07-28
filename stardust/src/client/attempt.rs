use std::{sync::mpsc::Receiver, net::SocketAddr, thread::JoinHandle};
use super::connection::ConnectionRejectionReason;

/// A running attempt to connect to a remote server.
#[derive(Debug)]
pub(super) struct ConnectionAttempt {
    receiver: Receiver<ConnectionAttemptUpdate>,
    remote: SocketAddr,
    handle: JoinHandle<()>,
}

/// The connection attempt has changed somehow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ConnectionAttemptUpdate {
    /// The connection timed out.
    TimedOut,
    /// The server rejected the client.
    Rejected(ConnectionRejectionReason),
    /// The server wants the client to wait for authentication to occur.
    ServerWaitAuth,
    /// The server accepted the client.
    Accepted,
}