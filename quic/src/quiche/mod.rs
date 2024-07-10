mod receiving;

use bevy::prelude::*;

pub(crate) fn setup(app: &mut App) {
    app.add_systems(PreUpdate, receiving::endpoints_receive_datagrams_system);
}