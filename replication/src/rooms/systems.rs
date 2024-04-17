use bevy::prelude::*;
use super::*;

pub(super) fn update_entity_cache(
    mut rooms: Query<(Entity, &mut NetworkRoom)>,
    filters: Query<(Entity, &NetworkRoomMembership), Changed<NetworkRoomMembership>>,
    removed: RemovedComponents<NetworkRoomMembership>,
) {
    rooms.par_iter_mut().for_each(|(room_entity, mut room)| {
        for (filter_entity, membership) in filters.iter() {
            let include = membership.memberships.includes(room_entity);

            if include {
                room.cache.insert(filter_entity);
            } else {
                room.cache.remove(&filter_entity);
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

pub(super) fn update_component_cache<T: Component>(
    mut rooms: Query<(Entity, &mut CachedMemberships<T>), With<NetworkRoom>>,
    filters: Query<(Entity, &NetworkRoomMembership<T>), Changed<NetworkRoomMembership<T>>>,
    removed: RemovedComponents<NetworkRoomMembership<T>>,
) {
    rooms.par_iter_mut().for_each(|(room_entity, mut room)| {
        for (filter_entity, membership) in filters.iter() {
            let include = membership.memberships.includes(room_entity);

            if include {
                room.cache.insert(filter_entity);
            } else {
                room.cache.remove(&filter_entity);
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