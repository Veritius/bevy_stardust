use std::{collections::HashMap, sync::{Exclusive, Mutex}, time::Instant};
use bytes::*;
use quinn_proto::*;
use bevy_ecs::prelude::*;

#[derive(Resource, Default)]
pub(crate) struct ConnectionHandleMap(pub HashMap<ConnectionHandle, Entity>);

/// A QUIC connection.
/// 
/// This component will be present even during a handshake.
/// Once the handshake is complete, the `NetworkPeer` component will be added.
#[derive(Component)]
pub struct QuicConnection {
    pub(crate) endpoint: Entity,
    pub(crate) handle: ConnectionHandle,
    pub(crate) inner: Exclusive<Connection>,
    pub(crate) events: Mutex<Vec<ConnectionEvent>>,
    pub(crate) disconnect_logged: bool,
}

impl QuicConnection {
    pub(crate) fn new(
        endpoint: Entity,
        handle: ConnectionHandle,
        connection: Connection
    ) -> Self {
        Self {
            endpoint,
            handle,
            inner: Exclusive::new(connection),
            events: Mutex::new(Vec::with_capacity(128)),
            disconnect_logged: false,
        }
    }

    /// Returns the entity ID of the endpoint performing IO for this connection.
    pub fn endpoint(&self) -> Entity {
        self.endpoint
    }

    /// Closes the connection.
    pub fn close(&mut self, reason: Bytes) {
        self.inner.get_mut().close(Instant::now(), VarInt::default(), reason)
    }
}

pub(super) fn update_handle_map_system(
    mut handle_map: ResMut<ConnectionHandleMap>,
    added: Query<(Entity, &QuicConnection), Added<QuicConnection>>,
) {
    for (id, comp) in added.iter() {
        handle_map.0.insert(comp.handle.clone(), id);
    }
}