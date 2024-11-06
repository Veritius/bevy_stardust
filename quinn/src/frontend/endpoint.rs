use bevy_ecs::prelude::*;
use crate::config::{ClientVerification, ServerAuthentication};

#[derive(Component)]
pub struct Endpoint {

}

impl Endpoint {
    pub fn new(
        auth: ServerAuthentication,
        verify: ClientVerification,
    ) -> Self {
        todo!()
    }
}