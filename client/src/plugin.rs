use bevy::prelude::*;
use bevy_stardust_shared::plugin::StardustSharedPlugin;

pub struct StardustClientPlugin {
    
}

impl Plugin for StardustClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(StardustSharedPlugin);
    }
}