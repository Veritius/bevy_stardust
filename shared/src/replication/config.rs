use std::marker::PhantomData;
use bevy::prelude::{Resource, Component};

#[derive(Resource)]
pub(crate) struct ComponentReplicationIdDispenser(pub u16);
impl ComponentReplicationIdDispenser {
    pub fn new() -> Self {
        Self(0)
    }

    pub fn new_id(&mut self) -> u16 {
        if self.0 == u16::MAX { panic!("Component replication ID limit exceeded!") }
        let id = self.0;
        self.0 += 1;
        return id
    }
}

#[derive(Resource)]
pub(crate) struct ReplicatedComponentData<T: Component> {
    pub replication_id: u16,
    pub replicate_by_default: bool,
    pub phantom: PhantomData<T>,
}