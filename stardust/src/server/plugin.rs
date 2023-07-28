use bevy::prelude::*;
use super::receive::AllChannelData;

pub struct StardustServerPlugin;

impl Plugin for StardustServerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AllChannelData::default());
    }
}