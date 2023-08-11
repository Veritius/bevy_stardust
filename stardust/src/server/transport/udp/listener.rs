use std::{net::{UdpSocket, SocketAddr}, io::ErrorKind, time::Instant};
use bevy::prelude::*;
use json::{JsonValue, object};
use once_cell::sync::Lazy;
use semver::Version;
use crate::{shared::hashdiff::UniqueNetworkHash, server::{clients::Client, settings::NetworkClientCap}};
use super::{policy::BlockingPolicy, STARDUST_UDP_VERSION_RANGE};

/// Minimum amount of bytes in a packet to even be bothered to be read.
const MINIMUM_PACKET_LENGTH: usize = 8;

/// Response sent to clients when the server is full.
static PLAYER_CAP_LIMIT_RESPONSE: Lazy<String> = Lazy::new(|| { object! { "response": "player_cap_reached" }.dump() });

/// Unfiltered socket for listening to UDP packets from unregistered peers.
#[derive(Resource)]
pub(super) struct UdpListener(pub UdpSocket);

impl UdpListener {
    pub fn new(port: u16) -> Self {
        let socket = UdpSocket::bind(format!("0.0.0.0:{}", port))
            .expect("Couldn't bind to port");
        
        socket.set_nonblocking(true).unwrap();

        Self(socket)
    }
}

#[derive(Default)]
pub(super) struct WaitingClients(Vec<UdpUnregisteredClient>);

impl WaitingClients {
    pub fn get_existing(&mut self, addr: SocketAddr) -> Option<(usize, &mut UdpUnregisteredClient)> {
        let mut index: Option<usize> = None;

        for (idx, cli) in self.0.iter().enumerate() {
            if cli.address == addr {
                index = Some(idx);
                break;
            }
        }

        if index.is_none() { return None; }
        let index = index.unwrap();

        Some((index, &mut self.0[index]))
    }
}

pub(super) struct UdpUnregisteredClient {
    address: SocketAddr,
    socket: UdpSocket,
    since: Instant,
    state: WaitingClientState,
}

pub(super) enum WaitingClientState {
    RemoveMeThisIsJustSoThereArentErrors,
}

pub(super) fn udp_listener_system(
    mut waiting: Local<WaitingClients>,
    existing: Query<(), With<Client>>,
    player_cap: Res<NetworkClientCap>,
    listener: Res<UdpListener>,
    hash: Res<UniqueNetworkHash>,
    policy: Option<Res<BlockingPolicy>>,
) {
    let mut buffer = [0u8; 1500];
    let players = existing.iter().len();
    let player_cap = player_cap.0 as usize;

    loop {
        // Check if we've run out of packets to read
        let packet = listener.0.recv_from(&mut buffer);
        if packet.as_ref().is_err_and(|e| e.kind() == ErrorKind::WouldBlock) { break; }
        let (octets, pkt_addr) = packet.unwrap();

        // Check packet size
        if octets < MINIMUM_PACKET_LENGTH { continue; }

        // Check the sending IP isn't blocked
        let blocked = policy
            .as_ref()
            .is_some_and(|v| v.addresses.contains(&pkt_addr.ip()));
        if blocked { continue }

        // Get relevant information
        let slice = &buffer[0..octets];
        let data = String::from_utf8_lossy(slice);

        // Check if we've already received a relevant packet from the client
        match waiting.get_existing(pkt_addr) {
            Some((index, client)) => {
                process_existing_client(&data, client);
            },
            None => {
                let new = process_new_client(
                    &data,
                    &listener.0,
                    players,
                    player_cap,
                    &hash,
                    pkt_addr
                );
                if new.is_some() { waiting.0.push(new.unwrap()); }
            },
        }
    }
}

fn process_existing_client(
    data: &str,
    client: &mut UdpUnregisteredClient,
) {
    match client.state {
        WaitingClientState::RemoveMeThisIsJustSoThereArentErrors => todo!(),
    }
}

fn process_new_client(
    data: &str,
    socket: &UdpSocket,
    active: usize,
    maximum: usize,
    hash: &UniqueNetworkHash,
    address: SocketAddr,
) -> Option<UdpUnregisteredClient> {
    // Parse JSON
    let json = json::parse(data);
    if json.is_err() { return None; }
    let json = json.unwrap();

    // Get fields from json
    let ver = json["version"].as_str();
    let pid = json["pid"].as_str();

    // Check correctness
    if ver == None || pid == None { return None; }
    let (ver, pid) = (ver.unwrap(), pid.unwrap());

    // Check the length of the fields to prevent amplification attacks
    if ver.len() < 1 { return None; }
    if pid.len() != 16 { return None; }

    // Check version value
    if let Ok(ver) = ver.parse::<Version>() {
        if !STARDUST_UDP_VERSION_RANGE.matches(&ver) {
            send_json(socket, address, object! {
                "response": "wrong_version",
                "requires": STARDUST_UDP_VERSION_RANGE.to_string()
            });
            return None;
        }
    } else {
        // Silently ignore them, sending a correct version is the responsibility of the client
        return None;
    }

    // Check game id hash
    if pid != hash.hex() {
        send_json(socket, address, object! {
            "response": "wrong_pid",
            "server_has": hash.hex()
        });
        return None;
    }

    // Check player limit
    if active >= maximum {
        let _ = socket.send_to(PLAYER_CAP_LIMIT_RESPONSE.as_bytes(), address);
        return None;
    }

    // Everything checks out
    return Some(UdpUnregisteredClient {
        address,
        socket: UdpSocket::bind(address).unwrap(),
        since: Instant::now(),
        state: WaitingClientState::RemoveMeThisIsJustSoThereArentErrors,
    })
}

fn send_json(socket: &UdpSocket, address: SocketAddr, json: JsonValue) {
    let b = json.dump();
    let _ = socket.send_to(b.as_bytes(), address);
}