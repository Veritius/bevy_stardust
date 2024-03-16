mod river;
mod packing;
mod frame;

use bevy_ecs::prelude::*;

#[derive(Component)]
pub(super) struct Established {

}

impl Established {
    pub fn new() -> Self {
        Self {
            
        }
    }
}