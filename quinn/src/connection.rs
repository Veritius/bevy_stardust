use std::time::Instant;
use bevy_ecs::{component::{ComponentHooks, StorageType}, prelude::*};
use quinn_proto::ConnectionHandle as QuinnHandle;
use crate::Endpoint;

/// A QUIC connection.
pub struct Connection(pub(crate) Box<ConnectionInner>);

impl Component for Connection {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_remove(|mut world, entity, _| {
            // Get this entity from the world.
            let this = match world.get_entity(entity) {
                Some(endpoint) => endpoint,
                None => return,
            };

            // Try to get the endpoint entity.
            let endpoint = this.get::<Connection>().unwrap().0.endpoint;
            let mut endpoint = match world.get_entity_mut(endpoint) {
                Some(endpoint) => endpoint,
                None => return,
            };

            // Try to access the endpoint component.
            if let Some(mut endpoint) = endpoint.get_mut::<Endpoint>() {
                // Inform the endpoint of the connection being closed
                unsafe { endpoint.0.inform_connection_close(entity) };
            }
        });
    }
}

pub(crate) struct ConnectionInner {
    handle: QuinnHandle,

    endpoint: Entity,

    connection: quinn_proto::Connection,
}

impl ConnectionInner {
    pub unsafe fn new(
        handle: QuinnHandle,
        endpoint: Entity,
        connection: quinn_proto::Connection,
    ) -> Self {
        Self {
            handle,
            endpoint,
            connection,
        }
    }

    pub fn close(
        &mut self
    ) {
        self.connection.close(
            Instant::now(),
            todo!(),
            todo!(),
        );
    }
}