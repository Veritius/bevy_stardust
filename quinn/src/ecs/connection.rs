use bevy_ecs::prelude::*;
use crate::backend::{connection::ConnectionHandle, endpoint::EndpointCreation};

#[derive(Component)]
pub struct Connection {
    inner: ConnectionInner,
}

enum ConnectionInner {
    Trying(Box<EndpointCreation>),
    Established(Box<ConnectionHandle>),
}