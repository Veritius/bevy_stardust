use std::time::Instant;
use bevy_ecs::{component::{ComponentHooks, StorageType}, prelude::*};
use quinn_proto::{ConnectionEvent, ConnectionHandle as QuinnHandle, EndpointEvent, Event as ApplicationEvent};
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
                unsafe { endpoint.inform_connection_close(entity) };
            }
        });
    }
}

pub(crate) struct ConnectionInner {
    handle: QuinnHandle,

    endpoint: Entity,

    connection: quinn_proto::Connection,

    statemachine: bevy_stardust_quic::Connection,
}

impl ConnectionInner {
    pub unsafe fn new(
        handle: QuinnHandle,
        endpoint: Entity,
        connection: quinn_proto::Connection,
        statemachine: bevy_stardust_quic::Connection,
    ) -> Self {
        Self {
            handle,
            endpoint,
            connection,
            statemachine,
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

    #[inline]
    pub fn handle(&self) -> QuinnHandle {
        self.handle
    }

    #[inline]
    pub fn quinn_handle_timeout(&mut self) {
        self.connection.handle_timeout(Instant::now());
    }

    #[inline]
    pub fn quinn_handle_event(
        &mut self,
        event: ConnectionEvent
    ) {
        self.connection.handle_event(
            event
        );
    }

    #[inline]
    pub fn quinn_poll_app(&mut self) -> Option<ApplicationEvent> {
        self.connection.poll()
    }

    #[inline]
    pub fn quinn_poll_end(&mut self) -> Option<EndpointEvent> {
        self.connection.poll_endpoint_events()
    }

    #[inline]
    pub fn handle_qio_timeout(&mut self) {
        self.statemachine.handle_timeout(Instant::now());
    }

    pub fn qio_poll(&mut self) -> Option<bevy_stardust_quic::ConnectionEvent> {
        self.statemachine.poll()
    }
}