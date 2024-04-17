mod queries;
mod systems;

pub use queries::NetChanged;

use std::marker::PhantomData;
use bevy::{prelude::*, ecs::component::*};
use crate::prelude::*;

/// Change detection tracking for network replicated types.
pub struct NetChanges<T> {
    pub(crate) ticks: ComponentTicks,
    phantom: PhantomData<T>,
}

impl<T: Resource> Resource for NetChanges<T> {}

impl<T: Component> Component for NetChanges<T> {
    type Storage = T::Storage;
}