use bevy_ecs::{prelude::*, system::SystemParam};
use crate::backend::executor::BackendExecutor;

/// [`SystemParam`] for working with QUIC endpoints and connections.
#[derive(SystemParam)]
pub struct QuicManager<'w, 's> {
    commands: Commands<'w, 's>,
    executor: Res<'w, BackendExecutor>,
}