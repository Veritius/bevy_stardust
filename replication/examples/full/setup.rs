use bevy::prelude::*;

pub(super) fn spawn_camera(
    mut commands: Commands
) {
    commands.spawn(Camera2dBundle::default());
}