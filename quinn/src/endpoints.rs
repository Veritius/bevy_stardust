use std::collections::BTreeSet;
use bevy::prelude::*;
use crate::connections::token::ConnectionOwnershipToken;

/// A QUIC endpoint using `quinn_proto`.
#[derive(Component)]
pub struct Endpoint {
    quic: quinn_proto::Endpoint,

    connections: BTreeSet<ConnectionOwnershipToken>,
}