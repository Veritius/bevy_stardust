use bevy_ecs::{prelude::*, system::SystemParam};
use super::polling::BackendExecutor;

/// [`SystemParam`] for working with QUIC connections.
#[derive(SystemParam)]
pub struct QuicManager<'w, 's> {
    executor: Res<'w, BackendExecutor>,
    commands: Commands<'w, 's>,
}