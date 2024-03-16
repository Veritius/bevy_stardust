mod river;
mod packing;
mod frame;
mod systems;

pub(crate) use systems::{
    established_breaking_system,
    established_packing_system,
};

use bevy_ecs::prelude::*;

#[derive(Component)]
pub(crate) struct Established {

}

impl Established {
    pub fn new() -> Self {
        Self {
            
        }
    }
}