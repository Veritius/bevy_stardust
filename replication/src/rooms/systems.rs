use bevy::prelude::*;
use super::*;

pub(super) fn update_entity_cache(
    mut rooms: Query<(Entity, &mut NetworkRoom)>,
    filters: Query<(Entity, &NetworkRoomMembership), Changed<NetworkRoomMembership>>,
    entities: Query<&Children, With<ReplicateEntity>>,
    removed: RemovedComponents<NetworkRoomMembership>,
) {
    rooms.par_iter_mut().for_each(|(room_entity, mut room)| {
        for (filter_entity, filter) in filters.iter() {
            let include = filter.filter.filter_inlined(room_entity);

            if include {
                room.cache.insert(filter_entity);

                for descendant in entities.iter_descendants(filter_entity) {
                    room.cache.insert(descendant);
                }
            } else {
                room.cache.remove(&filter_entity);

                for descendant in entities.iter_descendants(filter_entity) {
                    room.cache.remove(&descendant);
                }
            };
        }

        if let Some(events) = removed.events() {
            let mut reader = events.get_reader();
            let events = reader.read(events);
            for event in events {
                room.cache.remove(&event.clone().into());
            }
        }
    });
}

pub(super) fn update_component_cache<T: ReplicableComponent>(
    mut rooms: Query<(Entity, &mut CacheMemberships<T>), With<NetworkRoom>>,
    filters: Query<(Entity, &NetworkRoomMembership<T>), Changed<NetworkRoomMembership<T>>>,
    entities: Query<&Children, With<ReplicateEntity>>,
    removed: RemovedComponents<NetworkRoomMembership<T>>,
) {
    rooms.par_iter_mut().for_each(|(room_entity, mut room)| {
        for (filter_entity, filter) in filters.iter() {
            let include = filter.filter.filter_inlined(room_entity);

            if include {
                room.cache.insert(filter_entity);

                for descendant in entities.iter_descendants(filter_entity) {
                    room.cache.insert(descendant);
                }
            } else {
                room.cache.remove(&filter_entity);

                for descendant in entities.iter_descendants(filter_entity) {
                    room.cache.remove(&descendant);
                }
            };
        }

        if let Some(events) = removed.events() {
            let mut reader = events.get_reader();
            let events = reader.read(events);
            for event in events {
                room.cache.remove(&event.clone().into());
            }
        }
    });
}