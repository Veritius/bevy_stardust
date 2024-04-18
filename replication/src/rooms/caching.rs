use bevy::prelude::*;
use crate::prelude::*;
use super::*;

/// Caches room memberships for components of type `T` for faster access.
/// This will only apply to rooms with the [`CacheMemberships<T>`](CacheMemberships) component.
/// 
/// Entity memberships themselves are always cached.
pub struct CacheRoomMembershipsPlugin<T: Component>(PhantomData<T>);

impl<T: Component> Plugin for CacheRoomMembershipsPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, caching::update_component_cache::<T>
            .in_set(PostUpdateReplicationSystems::DetectChanges));
    }
}

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

/// Caches room memberships for the component `T`.
/// Improves iteration performance for entities with [`NetworkRoomMembership<T>`].
/// This comes at an additional cost of mutating or adding the [`NetworkRoomMembership<T>`] component.
/// 
/// This should be added to entities with the [`NetworkRoom`] component,
/// and not the [`NetworkRoomMembership`] component. Does nothing if
/// [`CacheRoomMembershipsPlugin`] isn't added.
#[derive(Default)]
pub struct CachedMemberships<T> {
    pub(super) cache: BTreeSet<Entity>,
    phantom: PhantomData<T>,
}

impl<T: Component> Component for CachedMemberships<T> {
    type Storage = T::Storage;
}