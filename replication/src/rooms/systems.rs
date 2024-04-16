use bevy::prelude::*;
use bevy_stardust::prelude::*;
use super::*;

pub(super) fn assign_identifiers_system(
    mut graph: ResMut<NetworkRoomGraph>,
    mut query: Query<(Entity, &mut NetworkRoom), Added<NetworkRoom>>,
) {
    for (entity, mut room) in query.iter_mut() {
        if room.id == None {
            let id = graph.graph.add_node(entity);
            room.id = Some(id);
        }
    }
}

pub(super) fn update_graph_links_system(
    mut graph: ResMut<NetworkRoomGraph>,
    mut rooms: Query<&NetworkRoom>,
) {

}