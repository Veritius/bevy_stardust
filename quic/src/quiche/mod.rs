mod receiving;
mod sending;

use bevy::prelude::*;
use quiche::ConnectionId;
use crate::plugin::QuicSystems;

pub(crate) fn setup(app: &mut App) {
    app.add_systems(PreUpdate, receiving::endpoints_receive_datagrams_system
        .in_set(QuicSystems::ReceivePackets));

    app.add_systems(PreUpdate, sending::endpoints_transmit_datagrams_system
        .in_set(QuicSystems::TransmitPackets));
}

fn issue_connection_id() -> ConnectionId<'static> {
    ConnectionId::from_vec(rand::random::<[u8; 16]>().into())
}