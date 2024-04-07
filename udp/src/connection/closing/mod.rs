mod systems;

use bytes::Bytes;
pub(crate) use systems::close_connections_system;

use bevy::prelude::*;
use crate::ConnectionDirection;

#[derive(Component)]
pub(crate) struct Closing {
    direction: ConnectionDirection,
    reason: Option<Bytes>,
}

impl Closing {
    pub fn new(direction: ConnectionDirection, reason: Option<Bytes>) -> Self {
        Self { direction, reason }
    }

    pub fn dir(&self) -> ConnectionDirection {
        self.direction
    }
}