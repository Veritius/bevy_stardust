use bevy::prelude::*;
use quinn::Connection;

#[derive(Component)]
pub(crate) struct QuicConnection(pub Connection);