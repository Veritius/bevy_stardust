use bevy_ecs::prelude::*;
use crate::backend::ConnectionHandle;

#[derive(Component)]
pub struct Connection {
    handle: ConnectionHandle,
}