use std::time::{Instant, Duration};
use bevy_ecs::prelude::*;
use bevy_stardust::connections::peer::NetworkPeer;
use bytes::{BufMut, Bytes, BytesMut};
use untrusted::*;
use crate::{appdata::{AppNetVersionWrapper, NetworkVersionData, BANNED_MINOR_VERSIONS, TRANSPORT_VERSION_DATA}, connection::{established::Established, reliability::{ReliabilityData, ReliablePacketHeader}, PotentialNewPeer}, endpoint::ConnectionOwnershipToken, packet::OutgoingPacket, Connection, ConnectionDirection, ConnectionState, Endpoint};
use crate::utils::IntegerFromByteSlice;
use super::{codes::HandshakeErrorCode, packets::{ClientFinalisePacket, ClientHelloPacket}};
use super::{codes::{response_code_from_int, HandshakeCode}, packets::ServerHelloPacket, HandshakeFailureReason, HandshakeState, Handshaking};

const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(15);
const RESEND_TIMEOUT: Duration = Duration::from_secs(5);

pub(crate) fn handshake_polling_system(
    appdata: Res<AppNetVersionWrapper>,
    commands: ParallelCommands,
    mut connections: Query<(Entity, &mut Connection, &mut Handshaking)>,
) {
    // IDK the performance of Instant::now(), but since it's a syscall,
    // err on the side of caution and do it once at the start of the system.
    let system_start_time = Instant::now();

    // Iterate connections in parallel
    connections.par_iter_mut().for_each(|(entity, mut connection, mut handshake)| {
        todo!();
    });
}

pub(crate) fn potential_new_peers_system(
    mut events: EventReader<PotentialNewPeer>,
    appdata: Res<AppNetVersionWrapper>,
    mut commands: Commands,
    mut endpoints: Query<&mut Endpoint>,
) {
    let mut ev_iter = events.read();
    while let Some(event) = ev_iter.next() {
        todo!();
    }
}

fn check_identity_match(
    us: &NetworkVersionData,
    them: &NetworkVersionData,
    banlist: &[u32],
    is_app: bool,
) -> Result<(), HandshakeErrorCode> {
    // Check the identity value
    if us.ident != them.ident {
        return Err(match is_app {
            true => HandshakeErrorCode::IncompatibleApplicationIdentifier,
            false => HandshakeErrorCode::IncompatibleTransportIdentifier,
        });
    }

    // Check the major version
    if us.major != them.major {
        return Err(match is_app {
            true => HandshakeErrorCode::IncompatibleApplicationMajorVersion,
            false => HandshakeErrorCode::IncompatibleTransportMajorVersion,
        });
    }

    // Check the minor version
    if banlist.contains(&them.minor) {
        return Err(match is_app {
            true => HandshakeErrorCode::IncompatibleApplicationMinorVersion,
            false => HandshakeErrorCode::IncompatibleTransportMinorVersion,
        });
    }

    Ok(())
}