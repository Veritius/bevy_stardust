use bevy::{ecs::system::SystemParam, prelude::*};

/// A [`SystemParam`] that contains information about the [`World`] required to create new endpoints and connections.
#[derive(SystemParam)]
pub struct QuinnManager<'w, 's> {
    pub(crate) commands: Commands<'w, 's>,

    #[cfg(debug_assertions)]
    pub(crate) world: bevy::ecs::world::WorldId,
}