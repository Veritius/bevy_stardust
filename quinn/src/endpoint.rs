use bevy::prelude::*;
use quinn_proto::Endpoint as QuinnEndpoint;

/// A QUIC endpoint.
#[derive(Component, Reflect)]
#[reflect(from_reflect = false, Component)]
pub struct Endpoint {
    #[reflect(ignore)]
    inner: Box<EndpointInner>,
}

struct EndpointInner {
    quinn: QuinnEndpoint,
}