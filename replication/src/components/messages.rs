use std::marker::PhantomData;
use bevy::prelude::*;
use crate::prelude::*;

#[derive(Resource)]
pub(super) struct ComponentSerialisationFunctions<T: Component> {
    pub fns: SerialisationFunctions<T>,
}

#[derive(Default)]
pub(super) struct ComponentReplicationMessages<T: Component>(PhantomData<T>);