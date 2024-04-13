use std::ops::Deref;
use bevy::{ecs::{component::*, system::SystemParam}, prelude::*};
use crate::prelude::*;

struct CurrentTick(pub Tick);

impl Deref for CurrentTick {
    type Target = Tick;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// SAFETY: This literally ignores everything mutable and just returns a number.
unsafe impl SystemParam for CurrentTick {
    type State = ();

    type Item<'world, 'state> = CurrentTick;

    fn init_state(
        _world: &mut World, 
        _system_meta: &mut bevy::ecs::system::SystemMeta
    ) -> Self::State {
        return ();
    }

    unsafe fn get_param<'world, 'state>(
        _state: &'state mut Self::State,
        _system_meta: &bevy::ecs::system::SystemMeta,
        _world: bevy::ecs::world::unsafe_world_cell::UnsafeWorldCell<'world>,
        change_tick: Tick,
    ) -> Self::Item<'world, 'state> {
        return Self(change_tick);
    }
}