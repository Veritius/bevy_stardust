use bevy_ecs::prelude::*;
use crate::backend::EndpointHandle;

#[derive(Component)]
pub struct Endpoint {
    handle: EndpointHandle,
}

impl Endpoint {
    pub fn new() -> Endpoint {
        todo!()
    }
}