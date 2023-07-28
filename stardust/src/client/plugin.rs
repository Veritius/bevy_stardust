use bevy::prelude::*;
use super::{receive::AllChannelData, connection::RemoteConnectionStatus};

pub struct StardustClientPlugin;

impl Plugin for StardustClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<RemoteConnectionStatus>();

        app.insert_resource(AllChannelData::default());
    }
}