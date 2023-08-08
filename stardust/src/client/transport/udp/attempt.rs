use std::{sync::mpsc::{Receiver, self}, net::SocketAddr, thread::{JoinHandle, self}};

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
    /// The server accepted the client.
    Accepted,
    /// The server rejected the client for an unknown reason.
    Rejected,
    /// The server didn't recognise the client's version of the Stardust UDP transport layer.
    WrongLayerVersion,
    /// The game version was invalid
    WrongGameVersion,
    /// The pid value was invalid
    WrongPid,
    /// The server was at capacity
    AtCapacity,
}

fn make_attempt(address: SocketAddr) -> ConnectionAttempt {
    let (sender, receiver) = mpsc::channel();
    let handle = thread::spawn(move || {
        let address = address.clone();
    });

    ConnectionAttempt {
        receiver,
        remote: address,
        handle,
    }
}