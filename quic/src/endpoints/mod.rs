use bevy::prelude::*;
use quinn_proto::Endpoint;

/// A QUIC endpoint.
#[derive(Component)]
pub struct QuicEndpoint {
    inner: Box<Endpoint>,
}