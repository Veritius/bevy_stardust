use bevy_ecs::{prelude::*, system::SystemParam};
use crate::backend::executor::BackendExecutor;

/// [`SystemParam`] for working with QUIC endpoints and connections.
#[derive(SystemParam)]
pub struct QuicManager<'w, 's> {
    pub(crate) commands: Commands<'w, 's>,
    pub(crate) executor: Res<'w, BackendExecutor>,
}