mod codes;
mod system;

pub(crate) use system::handshake_polling_system;

use bevy_stardust::connections::NetworkPeer;
use bytes::Bytes;
use std::{net::SocketAddr, time::Instant};
use bevy::prelude::*;
use crate::prelude::*;
use super::reliability::ReliabilityState;
use codes::HandshakeResponseCode;

#[derive(Component)]
#[component(storage = "SparseSet")]
pub(crate) struct Handshaking {
    started: Instant,
    reliability: ReliabilityState,
}