use bevy::prelude::Resource;

#[derive(Resource)]
pub enum NetworkState {
    Unconnected,
    Connecting,
    Connected
}