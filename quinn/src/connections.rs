use bevy::prelude::*;
use quinn_proto::ConnectionHandle;

/// A QUIC connection using `quinn_proto`.
#[derive(Component)]
pub struct Connection {
    endpoint: Entity,

    handle: ConnectionHandle,

    quinn: quinn_proto::Connection,
    qsm: bevy_stardust_quic::Connection,
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