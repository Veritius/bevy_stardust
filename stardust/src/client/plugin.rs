use std::collections::BTreeMap;

use bevy::prelude::*;

use super::receive::AllChannelData;

pub struct StardustClientPlugin;

impl Plugin for StardustClientPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AllChannelData(BTreeMap::new()));
    }
}