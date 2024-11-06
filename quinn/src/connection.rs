use bevy_ecs::prelude::*;
use crate::config::{ClientAuthentication, ServerVerification};
use super::endpoint::Endpoint;

#[derive(Component)]
pub struct Connection {

}

impl Connection {
    pub fn new(
        endpoint: &mut Endpoint,
        auth: ClientAuthentication,
        verify: ServerVerification,
    ) -> Self {
        todo!()
    }
}