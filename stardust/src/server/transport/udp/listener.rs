use std::net::{UdpSocket, SocketAddr};
use bevy::prelude::*;
use json::{JsonValue, object};
use once_cell::sync::Lazy;
use semver::Version;
use crate::{shared::{hashdiff::UniqueNetworkHash, channels::incoming::IncomingNetworkMessages}, server::{clients::Client, settings::NetworkClientCap, transport::udp::acks::ClientSequenceData}};
use super::{STARDUST_UDP_VERSION_RANGE, policy::BlockingPolicy, UdpClient, ports::PortBindings};

/// Minimum amount of bytes in a packet to be read at all.
const MINIMUM_PACKET_LENGTH: usize = 8;

/// Response sent to clients when the server is full.
static PLAYER_CAP_LIMIT_RESPONSE: Lazy<String> = Lazy::new(|| { object! { "response": "player_cap_reached" }.dump() });

/// Unfiltered socket for listening to UDP packets from unregistered peers.
#[derive(Resource)]
pub(super) struct UdpListener(pub UdpSocket);

impl UdpListener {
    pub fn new(address: std::net::IpAddr, port: u16) -> Self {
        let socket = UdpSocket::bind(SocketAddr::new(address, port))
            .expect("Failed to create UDP listener, is the port free?");
        socket.set_nonblocking(true).unwrap();
        info!("Created UdpListener at {}", socket.local_addr().unwrap());

        Self(socket)
    }
}

pub(super) fn udp_listener_system(
    mut commands: Commands,
    mut ports: ResMut<PortBindings>,
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
        let Ok((octets, pkt_addr)) = listener.0.recv_from(&mut buffer) else { break };

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

        // Process client data
        process_packet(
            &mut ports,
            &mut commands,
            &data,
            &listener.0,
            players,
            player_cap,
            &hash,
            pkt_addr,
        );
    }
}

/// Process packets sent to the listener port, registering clients if need be.
fn process_packet(
    bindings: &mut PortBindings,
    commands: &mut Commands,
    data: &str,
    socket: &UdpSocket,
    active: usize,
    maximum: usize,
    hash: &UniqueNetworkHash,
    address: SocketAddr,
) {
    // Parse JSON
    let json = json::parse(data);
    if json.is_err() { return; }
    let json = json.unwrap();

    // Get fields from json
    let req = json["request"].as_str(); // the request field isn't necessary but it makes amplification attacks that much harder
    let ver = json["version"].as_str();
    let pid = json["pid"].as_str();

    // Check correctness
    if req == None || ver == None || pid == None { return; }
    let (req, ver, pid) = (req.unwrap(), ver.unwrap(), pid.unwrap());

    // Check the length of the fields to prevent amplification attacks
    if req.len() < 3 { return; }
    if ver.len() < 1 { return; }
    if pid.len() != 16 { return; }

    // Check request type
    // Largely useless at the moment
    if req != "join" { return; }

    // Check version value
    if let Ok(ver) = ver.parse::<Version>() {
        if !STARDUST_UDP_VERSION_RANGE.matches(&ver) {
            send_json(socket, address, object! {
                "response": "wrong_version",
                "requires": STARDUST_UDP_VERSION_RANGE.to_string()
            });
            return;
        }
    } else {
        // Silently ignore them, sending a correct version is the responsibility of the client
        return;
    }

    // Check game id hash
    if pid != hash.hex() {
        send_json(socket, address, object! {
            "response": "wrong_pid",
            "server_has": hash.hex()
        });
        return;
    }

    // Check player limit
    if active >= maximum {
        let _ = socket.send_to(PLAYER_CAP_LIMIT_RESPONSE.as_bytes(), address);
        return;
    }

    // Create entity
    let ent_id = commands.spawn((
        Client::new(),
        UdpClient { address, hiccups: 0 },
        ClientSequenceData::new(),
        IncomingNetworkMessages::new(),
    )).id();

    // Bind a port to the client
    let port = bindings.add_client(ent_id);

    // Respond with acceptance
    send_json(socket, address, object! {
        "response": "accepted",
        "port": port
    });

    // Log join
    info!("New client joined via UDP from address {} and was assigned to entity id {:?}", address, ent_id);
}

fn send_json(socket: &UdpSocket, address: SocketAddr, json: JsonValue) {
    let b = json.dump();
    let _ = socket.send_to(b.as_bytes(), address);
}