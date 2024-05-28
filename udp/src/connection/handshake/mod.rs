mod codes;
mod system;

pub(crate) use system::handshake_polling_system;

use bevy_stardust::connections::NetworkPeer;
use std::{net::SocketAddr, time::Instant};
use bevy::prelude::*;
use crate::prelude::*;
use super::reliability::ReliabilityState;

#[derive(Component)]
pub(crate) struct Handshaking {
    started: Instant,
    reliability: ReliabilityState,
}

#[derive(Bundle)]
pub(crate) struct OutgoingHandshakeBundle {
    pub connection: Connection,
    handshake: Handshaking,
    peercomp: NetworkPeer,
}

impl OutgoingHandshakeBundle {
    pub fn new(
        owning_endpoint: Entity,
        remote_address: SocketAddr,
    ) -> Self {
        Self {
            connection: Connection::new(
                owning_endpoint,
                remote_address, 
                ConnectionDirection::Client,
            ),
            handshake: Handshaking {
                started: Instant::now(),
                reliability: ReliabilityState::new(),
            },
            peercomp: NetworkPeer::new(),
        }
    }
}