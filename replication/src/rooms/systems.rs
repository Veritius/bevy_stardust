use bevy::prelude::*;
use bevy_stardust::prelude::*;
use super::*;

pub(super) fn update_room_graph_system(
    mut graph: ResMut<NetworkRoomGraph>,
    rooms: Query<(&NetworkRoom, &NetworkGroup)>,
) {

}