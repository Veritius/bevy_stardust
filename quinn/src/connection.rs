use bevy::prelude::*;
use bevy_stardust::prelude::*;
use bevy_stardust_quic::Connection as ConnectionState;
use quinn::{Connecting, Connection};

/// Represents one Quinn connection.
#[derive(Component)]
pub struct QuinnConnection {
    qn_state: ConnectionInner,
    sp_state: Box<ConnectionState>,
}

impl QuinnConnection {
    pub(crate) fn connecting(connecting: Connecting) -> Self {
        Self {
            qn_state: todo!(),
            sp_state: Box::new(ConnectionState::new()),
        }
    }
}

struct ConnectionInner {

}

pub(crate) fn message_recv_system(
    mut query: Query<(&mut QuinnConnection, &mut PeerMessages<Incoming>)>,
) {
    query.par_iter_mut().for_each(|(mut connection, mut messages)| {
        todo!()
    });
}

pub(crate) fn message_send_system(
    mut query: Query<(&mut QuinnConnection, &PeerMessages<Outgoing>)>,
) {
    query.par_iter_mut().for_each(|(mut connection, mut messages)| {
        todo!()
    });
}