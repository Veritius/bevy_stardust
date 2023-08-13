use std::{time::{Duration, Instant}, net::{SocketAddr, UdpSocket}, ops::Deref};
use bevy::prelude::*;
use json::object;
use semver::Version;
use crate::{shared::hashdiff::UniqueNetworkHash, client::{connection::RemoteConnectionStatus, peers::Server, transport::udp::RemoteServerUdpSocket}};

/// The version of the transport layer.
const TRANSPORT_LAYER_VERSION: Version = Version::new(0, 0, 0);
/// Amount of time the client should wait for a server response before giving up.
const RESPONSE_TIMEOUT_DURATION: Duration = Duration::from_secs(15);
/// Time between attempts to resend packets if there is no response.
const PACKET_RESEND_DURATION: Duration = Duration::from_nanos(1_000_000_000 / 1);

/// Add to try and connect to a remote server.
/// 
/// The port in the `SocketAddr` should be the **listen port** the server is using, not the active port.
#[derive(Resource)]
pub(super) struct ConnectToRemoteUdp(pub SocketAddr);

pub(super) fn connection_attempt_system(
    mut commands: Commands,
    state: Res<State<RemoteConnectionStatus>>,
    mut next: ResMut<NextState<RemoteConnectionStatus>>,
    hash: Res<UniqueNetworkHash>,

    target: Option<Res<ConnectToRemoteUdp>>,
    mut started: Local<Option<Instant>>,
    mut tsocket: Local<Option<UdpSocket>>,
    mut last_sent: Local<Option<Instant>>,
) {
    // Check if there is a target to join
    match (&target, *started) {
        (Some(target), None) => {
            next.set(RemoteConnectionStatus::Connecting);
            *started = Some(Instant::now());
            let skt = UdpSocket::bind(format!("0.0.0.0:0")).unwrap();
            let _ = skt.set_nonblocking(true);
            let _ = skt.connect(target.0);
            info!("Trying to join remote peer {} over socket {}", target.0, skt.local_addr().unwrap());
            *tsocket = Some(skt);
        },
        (Some(_), Some(_)) => {},
        _ => { return; }
    }

    let target_just_added = target.as_ref().is_some_and(|target| target.is_added());

    // Sanity checks to prevent multiple transport layers potentially running at the same time
    if *state == RemoteConnectionStatus::Connected {
        panic!("Connect resource was added but state is Connected. Is there multiple transport layers added to the App?");
    }
    if *state == RemoteConnectionStatus::Connecting && target_just_added {
        panic!("UDP connection resource was only just added but state was already Connecting. Is there multiple transport layers added to the App?");
    }

    // Read socket for any responses
    let Some(socket) = tsocket.deref() else { panic!() };
    if target_just_added {
        let mut failed = false;
        let mut buffer = [0u8; 1500];

        loop {
            // Receive octets and parse json
            let Ok(octets) = socket.recv(&mut buffer) else { break };
            let string = String::from_utf8_lossy(&buffer[0..octets]);
            let parsed = json::parse(&string);
            if parsed.is_err() { continue; } // invalid json, continue
            let parsed = parsed.unwrap();

            // Read json
            let resp = parsed["response"].as_str();
            match resp {
                Some("accepted") => {
                    // Check port value
                    let port = parsed["port"].as_u16();
                    if port.is_none() {
                        failed = true;
                        next.set(RemoteConnectionStatus::Unconnected);
                        break;
                    }
                    let port = port.unwrap();

                    // Create socket
                    let new_address = SocketAddr::new(target.unwrap().0.ip(), port);
                    let new_socket = UdpSocket::bind(new_address)
                        .expect("Unable to bind to SocketAddr despite previously communicating with the server");

                    // Log acceptance
                    info!("Accepted by remote server {}", new_address);

                    // Modify world
                    commands.spawn(Server);
                    commands.insert_resource(RemoteServerUdpSocket(new_socket));
                    commands.remove_resource::<ConnectToRemoteUdp>();
                    next.set(RemoteConnectionStatus::Connected);
                    (*tsocket, *started, *last_sent) = (None, None, None);
                    return;
                },
                Some("denied") => {
                    info!("Denied by remote server {}", target.as_ref().unwrap().0);
                    failed = true;
                    break;
                },
                None | _ => {
                    info!("Remote server {} sent invalid response", target.as_ref().unwrap().0);
                    break;
                },
            };
        }

        if failed {
            // Failed to connect
            commands.remove_resource::<ConnectToRemoteUdp>();
            next.set(RemoteConnectionStatus::Unconnected);
            (*tsocket, *started, *last_sent) = (None, None, None);
            return;
        }
    }

    // Check for timeout
    if started.unwrap().duration_since(Instant::now()) > RESPONSE_TIMEOUT_DURATION {
        info!("Timed out from connecting to remote server {}", target.as_ref().unwrap().0);
        commands.remove_resource::<ConnectToRemoteUdp>();
        (*tsocket, *started, *last_sent) = (None, None, None);
        return;
    }

    // Send a packet to try and join
    if last_sent.is_none() || last_sent.is_some_and(|last_sent| last_sent.duration_since(Instant::now()) >= PACKET_RESEND_DURATION) {
        // Create json data
        let json = object! {
            "request": "join",
            "version": TRANSPORT_LAYER_VERSION.to_string(),
            "pid": hash.hex()
        };

        // Send json data
        let data = json.dump();
        let _ = socket.send(data.as_bytes());
        *last_sent = Some(Instant::now());
    }
}