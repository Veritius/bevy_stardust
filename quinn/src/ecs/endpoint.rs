use bevy_ecs::prelude::*;
use crate::backend::endpoint::{EndpointCreation, EndpointHandle};

#[derive(Component)]
pub struct Endpoint {
    inner: EndpointInner,
}

enum EndpointInner {
    Trying(Box<EndpointCreation>),
    Established(Box<EndpointHandle>),
}