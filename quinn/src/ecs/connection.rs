use bevy_ecs::prelude::*;
use crate::backend::connection::{ConnectionRef, MessageHandle};

/// A QUIC connection.
#[derive(Component)]
pub struct Connection {
    inner: ConnectionRef,
    handle: MessageHandle,
}