use std::{time::{Duration, Instant}, net::{SocketAddr, UdpSocket}, ops::Deref};
use bevy::prelude::*;
use json::object;
use semver::Version;
use crate::{shared::hashdiff::UniqueNetworkHash, client::connection::RemoteConnectionStatus};

/// Turns a `Local<Option<T>>` into a `&T`. Panics if it's `None`.
/// 
/// Resolves into a let statement.
macro_rules! extract_local {
    ($i: ident) => {
        let Some($i) = $i.deref() else { panic!() };
    };
}

/// The version of the transport layer.
const TRANSPORT_LAYER_VERSION: Version = Version::new(0, 1, 0);
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
    state: Res<State<RemoteConnectionStatus>>,
    hash: Res<UniqueNetworkHash>,

    target: Option<Res<ConnectToRemoteUdp>>,
    mut started: Local<Option<Instant>>,
    mut socket: Local<Option<UdpSocket>>,
    mut last_sent: Local<Option<Instant>>,
) {
    // Check if there is a target to join
    match (&target, *started) {
        (None, None) => { return; },
        (Some(target), None) => {
            *started = Some(Instant::now());
            *socket = Some(UdpSocket::bind(target.0).unwrap());
        },
        _ => { return; }
    }

    // Sanity checks to prevent multiple transport layers potentially running at the same time
    if *state == RemoteConnectionStatus::Connected {
        panic!("Connect resource was added but state is Connected. Is there multiple transport layers added to the App?");
    }
    if *state == RemoteConnectionStatus::Connecting && target.is_some_and(|target| target.is_added()) {
        panic!("UDP connection resource was only just added but state was already Connecting. Is there multiple transport layers added to the App?");
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
        extract_local!(socket);
        let _ = socket.send(data.as_bytes());
        *last_sent = Some(Instant::now());
    }
}