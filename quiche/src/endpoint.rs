use std::net::UdpSocket;
use bevy::prelude::*;

/// A QUIC endpoint.
#[derive(Component, Reflect)]
#[reflect(from_reflect = false, Component)]
pub struct Endpoint {
    #[reflect(ignore)]
    inner: Box<EndpointInner>,
}

struct EndpointInner {
    socket: UdpSocket,
}