use std::{collections::BTreeSet, net::UdpSocket};
use bevy::prelude::*;
use crate::connections::token::ConnectionOwnershipToken;

/// A QUIC endpoint using `quinn_proto`.
#[derive(Component)]
pub struct Endpoint {
    socket: UdpSocket,

    quinn: quinn_proto::Endpoint,

    connections: BTreeSet<ConnectionOwnershipToken>,
}