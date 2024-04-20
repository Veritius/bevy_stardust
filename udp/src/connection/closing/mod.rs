mod systems;

pub(crate) use systems::close_connections_system;

use bytes::Bytes;
use bevy::prelude::*;
use crate::prelude::*;

pub const CLOSE_REASON_UNSPECIFIED: Bytes = Bytes::from_static("No reason given".as_bytes());

pub(super) struct CloseOrder {
    pub hard: bool,
    pub reason: Bytes,
}

#[derive(Component)]
pub(crate) struct Closing;