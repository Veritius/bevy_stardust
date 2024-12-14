use bevy_ecs::prelude::*;
use crate::backend::ConnectionHandle;

#[derive(Component)]
pub struct Connection {
    handle: ConnectionHandle,
}

impl Connection {
    pub fn new(
        endpoint: &super::endpoint::Endpoint,
    ) -> Connection {
        todo!()
    }
}