use std::{sync::mpsc::{Receiver, self}, net::{SocketAddr, TcpListener, TcpStream}, thread::{JoinHandle, self}, time::Duration, io::{ErrorKind, Write}};
use json::object;
use semver::Version;

const LAYER_VERSION: Version = Version::new(0, 1, 0);

#[derive(Debug)]
pub(super) struct ConnectionAttemptConfig {
    pub target: SocketAddr,
    pub version: Version,
    pub pid: u64,
}

/// A running attempt to connect to a remote server.
#[derive(Debug)]
pub(super) struct ConnectionAttempt {
    receiver: Receiver<ConnectionAttemptResult>,
    remote: SocketAddr,
}

/// The connection attempt has changed somehow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ConnectionAttemptResult {
    /// The TcpStream encountered a problem.
    TcpError,
    /// The connection timed out.
    TimedOut,
    /// The server accepted the client.
    Accepted,
    /// The server rejected the client for an unknown reason.
    Rejected,
    /// The server didn't recognise the client's version of the Stardust UDP transport layer.
    WrongLayerVersion,
    /// The game version was invalid.
    WrongGameVersion,
    /// The pid value was invalid.
    WrongPid,
    /// The server was at capacity.
    AtCapacity,
}

pub(super) fn make_attempt(config: ConnectionAttemptConfig) -> ConnectionAttempt {
    let remote = config.target.clone();

    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        // Move/clone some values
        let config = config;
        let sender = sender;

        // Create TCP listener
        let tcp = TcpStream::connect_timeout(&config.target, Duration::from_secs(20));
        if tcp.as_ref().is_err_and(|e| e.kind() == ErrorKind::TimedOut) {
            let _ = sender.send(ConnectionAttemptResult::TimedOut);
            return;
        }
        if tcp.as_ref().is_err() {
            let _ = sender.send(ConnectionAttemptResult::TcpError);
            return;
        }
        let mut tcp = tcp.unwrap();

        let initial_json = object! {
            "layer_version": LAYER_VERSION.to_string(),
            "game_version": config.version.to_string(),
            "pid": format!("{:X}", config.pid)
        };
        let _ = tcp.write(initial_json.dump().as_bytes());
        let _ = tcp.flush();
    });

    ConnectionAttempt {
        receiver,
        remote,
    }
}