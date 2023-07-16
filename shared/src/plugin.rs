use bevy::prelude::{Plugin, App};

use crate::protocol::ProtocolBuilder;

pub struct StardustSharedPlugin;
impl Plugin for StardustSharedPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ProtocolBuilder::default());
    }
}