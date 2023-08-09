use std::{net::{SocketAddr, TcpStream, Shutdown}, thread::{self, JoinHandle}, time::{Duration, Instant}, io::{ErrorKind, Write, Read}};
use json::{object, JsonValue};
use semver::Version;

/// The version of the transport layer.
const LAYER_VERSION: Version = Version::new(0, 1, 0);
/// Time before the TCP connection attempt stops.
const TCP_TIMEOUT_DURATION: Duration = Duration::from_secs(15);
/// Amount of time the client should wait for a server response before closing.
const RESPONSE_TIMEOUT_DURATION: Duration = Duration::from_secs(15);

#[derive(Debug)]
pub(super) struct ConnectionAttemptConfig {
    pub target: SocketAddr,
    pub version: Version,
    pub pid: u64,
}

/// A running attempt to connect to a remote server.
#[derive(Debug)]
pub(super) struct ConnectionAttempt {
    handle: JoinHandle<ConnectionAttemptResult>,
    remote: SocketAddr,
}

/// The connection attempt has changed somehow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ConnectionAttemptResult {
    /// The TcpStream encountered a problem.
    TcpError,
    /// The connection timed out.
    TimedOut,
    /// The server sent an invalid response.
    BadServerResponse,
    /// The server accepted the client.
    Accepted,
    /// The server rejected the client for an unknown reason.
    Rejected,
    /// The server didn't recognise the client's version of the Stardust UDP transport layer.
    WrongLayerVersion(Option<String>),
    /// The pid value was invalid.
    WrongPid(Option<String>),
    /// The server was at capacity.
    ServerAtCapacity,
}

pub(super) fn make_attempt(config: ConnectionAttemptConfig) -> ConnectionAttempt {
    let remote = config.target.clone();

    let handle = thread::spawn(move || {
        // Move/clone some values
        let config = config;

        let response = ConnectionAttemptResult::Rejected;

        response
    });

    ConnectionAttempt {
        handle,
        remote,
    }
}