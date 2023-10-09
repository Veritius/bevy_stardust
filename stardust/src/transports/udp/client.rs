//! Logic for clients to try and join a server.

use std::{net::SocketAddr, time::{Duration, Instant}};
use bevy::prelude::*;
use json::object;
use crate::{transports::udp::TRANSPORT_LAYER_VERSION_STR, protocol::UniqueNetworkHash, prelude::{NetworkPeer, server::Client, client::Server}};
use super::{ports::PortBindings, UdpTransportState, PACKET_MAX_BYTES, peer::UdpPeer};

/// Add to try and connect to a remote server.
#[derive(Resource)]
pub(super) struct TryConnectToRemote {
    pub address: SocketAddr,
    pub timeout: Duration,
}

pub(super) fn client_acceptance_system(
    mut ports: ResMut<PortBindings>,
    attempt: Res<TryConnectToRemote>,
    hash: Res<UniqueNetworkHash>,
    mut started: Local<Option<Instant>>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<UdpTransportState>>,
) {
    let socket = ports.iter().nth(0).unwrap().1;

    // Try to receive some bytes
    let mut buffer = [0u8; PACKET_MAX_BYTES];

    let shutdown = loop {
        let (octets, origin) = match socket.recv_from(&mut buffer) {
            Ok(n) => n,
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // Nothing received
    
                // Check timeout
                let mut set_started = None;
                match *started {
                    Some(started) => {
                        if Instant::now().duration_since(started) > attempt.timeout {
                            commands.remove_resource::<TryConnectToRemote>();
                            set_started = None;
                        }
                    },
                    None => {
                        set_started = Some(Instant::now());
                    },
                }
                *started = set_started;
    
                // Send a packet
                let buf = object! {
                    "request": "join",
                    "layer_version": TRANSPORT_LAYER_VERSION_STR,
                    "pid": hash.hex(),
                }.dump();
                let _ = socket.send_to(buf.as_bytes(), attempt.address);
                break false;
            }
            Err(e) => {
                // Something went wrong
                error!("Failed to connect to remote server: IO error while reading UDP socket {:?}: {}", socket.local_addr().unwrap(), e);
                break true;
            },
        };

        // Check packet origin
        if origin != attempt.address { continue; }

        // Try to parse the packet
        let string = std::str::from_utf8(&buffer[3..octets]);
        let string = if string.is_err() {
            info!("Failed to connect to remote server: response packet was invalid string");
            break true;
        } else { string.unwrap() };
        let json = json::parse(string);
        let json = if json.is_err() {
            info!("Failed to connect to remote server: response packet was unparseable");
            break true;
        } else { json.unwrap() };

        let response = match &json["response"] {
            json::JsonValue::Short(val) => val.as_str(),
            json::JsonValue::String(val) => val.as_str(),
            _ => {
                info!("Failed to connect to remote server: response packet had no response key");
                break true;
            }
        };

        match response {
            "accepted" => {
                // Get port number
                let port = match &json["port"] {
                    json::JsonValue::Number(number) => {
                        match TryInto::<u16>::try_into(*number) {
                            Ok(val) => {
                                val
                            },
                            Err(_) => {
                                info!("Failed to connect to remote server: port number invalid");
                                break true;
                            },
                        }
                    },
                    _ => {
                        info!("Failed to connect to remote server: no port field in acceptance message");
                        break true;
                    }
                };

                let entity = commands.spawn((
                    NetworkPeer { connected: Instant::now() },
                    UdpPeer { address: attempt.address.clone(), hiccups: 0 },
                    Server,
                )).id();
                ports.add_client(entity);
                next_state.set(UdpTransportState::Client);
                break false;
            },
            // TODO: this logging isn't very necessary
            "denied" => {
                info!("Rejected by remote server: no reason given");
                break true;
            },
            "ip_blocked" => {
                info!("Rejected by remote server: ip blocked (banned?)");
                break true;
            },
            "player_cap_reached" => {
                info!("Rejected by remote server: too many players");
                break true;
            },
            "wrong_transport_version" => {
                info!("Rejected by remote server: wrong transport version");
                break true;
            },
            "wrong_pid" => {
                info!("Rejected by remote server: server's protocol id didn't match");
                break true;
            },
            _ => {
                info!("Failed to connect to remote server: didn't understand response");
                break true;
            },
        }
    };

    if shutdown {
        commands.remove_resource::<TryConnectToRemote>();
        *started = None;
    }
}
