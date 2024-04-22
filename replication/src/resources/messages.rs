use std::marker::PhantomData;
use bevy::prelude::*;
use crate::prelude::*;

#[derive(Resource)]
pub(super) struct ResourceSerialisationFunctions<T: Resource> {
    pub fns: SerialisationFunctions<T>,
}

#[derive(Default)]
pub(super) struct ResourceReplicationMessages<T: Resource>(PhantomData<T>);