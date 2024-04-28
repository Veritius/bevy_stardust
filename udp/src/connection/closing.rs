use bevy::prelude::*;
use bevy_stardust::prelude::*;
use super::{handshake::Handshaking, Connection};

pub(super) fn close_events_system(
    mut events: EventReader<DisconnectPeerEvent>,
    mut connections: Query<(&mut Connection, Option<&mut Handshaking>, Option<&mut Connection>)>,
) {

}