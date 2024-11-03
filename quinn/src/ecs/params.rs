use bevy_ecs::{entity::Entities, prelude::*, system::SystemParam};
use crate::config::*;
use super::polling::BackendExecutor;

/// [`SystemParam`] for working with QUIC endpoints and connections.
#[derive(SystemParam)]
pub struct QuicManager<'w, 's> {
    commands: Commands<'w, 's>,
    entities: &'w Entities,
    executor: Res<'w, BackendExecutor>,
}

impl<'w, 's> QuicManager<'w, 's> {

}