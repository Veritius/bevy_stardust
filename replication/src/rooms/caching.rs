use bevy::prelude::*;
use crate::prelude::*;
use super::*;

/// Caches room memberships for components of type `T` for faster access.
/// This will only apply to rooms with the [`CacheMemberships<T>`](CacheMemberships) component.
/// 
/// Entity memberships themselves are always cached.
#[derive(Default)]
pub struct CacheRoomMembershipsPlugin<T: Component>(PhantomData<T>);

impl<T: Component> Plugin for CacheRoomMembershipsPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, caching::cache_update_system::<CachedMemberships<T>, T>
            .in_set(PostUpdateReplicationSystems::DetectChanges));
    }
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

pub(super) fn cache_update_system<C: MembershipCache + Component, M: Component>(
    mut rooms: Query<(Entity, &mut C), With<NetworkRoom>>,
    members: Query<(Entity, &NetworkRoomMembership<M>), Changed<NetworkRoomMembership<M>>>,
    removed: RemovedComponents<NetworkRoomMembership<M>>,
) {
    rooms.par_iter_mut().for_each(|(room_entity, mut room)| {
        for (member_entity, member_data) in members.iter() {
            let include = member_data.memberships.includes(room_entity);

            if include {
                room.insert(member_entity);
            } else {
                room.remove(member_entity);
            };
        }

        if let Some(events) = removed.events() {
            let mut reader = events.get_reader();
            let events = reader.read(events);
            for event in events {
                room.remove(event.clone().into());
            }
        }
    });
}

pub(super) trait MembershipCache {
    fn insert(&mut self, entity: Entity);
    fn remove(&mut self, entity: Entity);
}

impl MembershipCache for NetworkRoom {
    #[inline]
    fn insert(&mut self, entity: Entity) {
        self.cache.insert(entity);
    }

    #[inline]
    fn remove(&mut self, entity: Entity) {
        self.cache.remove(&entity);
    }
}

impl<T> MembershipCache for CachedMemberships<T> {
    #[inline]
    fn insert(&mut self, entity: Entity) {
        self.cache.insert(entity);
    }

    #[inline]
    fn remove(&mut self, entity: Entity) {
        self.cache.remove(&entity);
    }
}