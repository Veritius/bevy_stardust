mod receiving;

use bevy::prelude::*;

use crate::plugin::QuicSystems;

pub(crate) fn setup(app: &mut App) {
    app.add_systems(PreUpdate, receiving::endpoints_receive_datagrams_system
        .in_set(QuicSystems::ReceivePackets));
}