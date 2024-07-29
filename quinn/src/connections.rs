use bevy::prelude::*;
use quinn_proto::{ConnectionEvent as QuinnConnectionEvent, ConnectionHandle, EndpointEvent};

/// A QUIC connection using `quinn_proto`.
/// 
/// # Safety
/// This component must not be moved from the [`World`] it was originally added to.
#[derive(Component)]
pub struct Connection {
    endpoint: Entity,

    handle: ConnectionHandle,

    quinn: quinn_proto::Connection,
    qsm: bevy_stardust_quic::Connection,

    #[cfg(debug_assertions)]
    world: bevy::ecs::world::WorldId,
}

impl Connection {
    pub(crate) fn handle_event(&mut self, event: QuinnConnectionEvent) {
        self.quinn.handle_event(event);
    }

    pub(crate) fn poll_endpoint_events(&mut self) -> Option<EndpointEvent> {
        self.quinn.poll_endpoint_events()
    }
}

#[cfg(debug_assertions)]
pub(crate) fn safety_check_system(
    world: bevy::ecs::world::WorldId,
    connections: Query<&Connection>,
) {
    for connection in &connections {
        assert_eq!(connection.world, world,
            "A Connection had a world ID different from the one it was created in. This is undefined behavior!");
    }
}

pub(crate) mod token {
    use super::*;

    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
    #[repr(transparent)]
    pub(crate) struct ConnectionOwnershipToken(Entity);

    impl ConnectionOwnershipToken {
        /// Creates a new [`ConnectionOwnershipToken`] from an [`Entity`] identifier.
        /// 
        /// # SAFETY
        /// There must only be one token for one `id` value in the `World`.
        pub unsafe fn new(id: Entity) -> Self {
            Self(id)
        }

        #[inline]
        pub fn inner(&self) -> Entity {
            self.0
        }
    }

    impl PartialEq<Entity> for ConnectionOwnershipToken {
        #[inline]
        fn eq(&self, other: &Entity) -> bool {
            self.0.eq(other)
        }
    }

    impl PartialEq<ConnectionOwnershipToken> for Entity {
        #[inline]
        fn eq(&self, other: &ConnectionOwnershipToken) -> bool {
            self.eq(&other.0)
        }
    }

    impl From<&ConnectionOwnershipToken> for Entity {
        #[inline]
        fn from(value: &ConnectionOwnershipToken) -> Self {
            value.inner()
        }
    }
}