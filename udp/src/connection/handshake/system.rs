use std::time::{Instant, Duration};
use bevy_ecs::prelude::*;
use crate::{appdata::{AppNetVersionWrapper, NetworkVersionData}, connection::{Connection, PotentialNewPeer}, Endpoint};
use super::codes::HandshakeResponseCode;
use super::Handshaking;

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
) -> Result<(), HandshakeResponseCode> {
    // Check the identity value
    if us.ident != them.ident {
        return Err(match is_app {
            true => HandshakeResponseCode::IncompatibleApplicationIdentifier,
            false => HandshakeResponseCode::IncompatibleTransportIdentifier,
        });
    }

    // Check the major version
    if us.major != them.major {
        return Err(match is_app {
            true => HandshakeResponseCode::IncompatibleApplicationMajorVersion,
            false => HandshakeResponseCode::IncompatibleTransportMajorVersion,
        });
    }

    // Check the minor version
    if banlist.contains(&them.minor) {
        return Err(match is_app {
            true => HandshakeResponseCode::IncompatibleApplicationMinorVersion,
            false => HandshakeResponseCode::IncompatibleTransportMinorVersion,
        });
    }

    Ok(())
}