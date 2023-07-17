use bevy::prelude::{Plugin, App};
use crate::{protocol::ProtocolBuilder};

pub struct StardustSharedPlugin;
impl Plugin for StardustSharedPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ProtocolBuilder::default());
    }

    fn finish(&self, app: &mut App) {
        let builder = app.world.remove_resource::<ProtocolBuilder>()
            .expect("Builder should have been present").build();
        app.world.insert_resource(builder);
    }
}