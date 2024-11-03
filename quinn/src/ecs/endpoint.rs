use bevy_ecs::prelude::*;
use crate::backend::endpoint::EndpointRef;

#[derive(Component)]
pub struct Endpoint {
    inner: EndpointRef,
}