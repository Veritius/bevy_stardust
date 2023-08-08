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

        // Create TCP stream
        let tcp = TcpStream::connect_timeout(&config.target, TCP_TIMEOUT_DURATION);
        if tcp.as_ref().is_err_and(|e| e.kind() == ErrorKind::TimedOut) {
            return ConnectionAttemptResult::TimedOut
        }
        if tcp.as_ref().is_err() {
            return ConnectionAttemptResult::TcpError
        }
        let mut tcp = tcp.unwrap();

        // Send data about the client in JSON
        let initial_json = object! {
            "layer_version": LAYER_VERSION.to_string(),
            "game_version": config.version.to_string(),
            "pid": format!("{:X}", config.pid)
        };
        let _ = tcp.write(initial_json.dump().as_bytes());
        let _ = tcp.flush();

        // Used for timeout logic
        let started = Instant::now();

        // Used for return
        let thread_response: ConnectionAttemptResult;

        // Wait for new packets
        let mut buffer = [0u8; 1500];
        loop {
            if let Ok(bytes) = tcp.read(&mut buffer) {
                let str = String::from_utf8_lossy(&buffer[0..bytes]);
                let json = json::parse(&str);
                if let Ok(json) = json {
                    if let Some(response) = json["response"].as_str() {
                        // Helper function
                        fn get_string_from_json(json: JsonValue, key: &str) -> Option<String> {
                            match json[key].as_str() {
                                Some(s) => Some(s.to_string()),
                                None => None,
                            }
                        }

                        // Match response code
                        match response {
                            "wrong_layer_version" => {
                                let ver = get_string_from_json(json, "range");
                                thread_response = ConnectionAttemptResult::WrongLayerVersion(ver);
                                break;
                            },
                            "wrong_pid" => {
                                let pid = get_string_from_json(json, "srv_pid");
                                thread_response = ConnectionAttemptResult::WrongPid(pid);
                                break;
                            },
                            "at_capacity" => {
                                thread_response = ConnectionAttemptResult::ServerAtCapacity;
                                break;
                            },
                            "retry" => todo!(),
                            "accepted" => {
                                thread_response = ConnectionAttemptResult::Accepted;
                                break;
                            },
                            "denied" => {
                                thread_response = ConnectionAttemptResult::Rejected;
                                break;
                            },
                            _ => {
                                thread_response = ConnectionAttemptResult::BadServerResponse;
                                break;
                            }
                        }
                    } else {
                        thread_response = ConnectionAttemptResult::BadServerResponse;
                        break;
                    }
                } else {
                    thread_response = ConnectionAttemptResult::BadServerResponse;
                    break;
                }
            } else {
                // Check for timeout
                if started.elapsed() > RESPONSE_TIMEOUT_DURATION {
                    thread_response = ConnectionAttemptResult::TimedOut;
                    break;
                }
            }
        }

        // Cleanly shut down connection and return
        let _ = tcp.shutdown(Shutdown::Both);
        thread_response
    });

    ConnectionAttempt {
        handle,
        remote,
    }
}