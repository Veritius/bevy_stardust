use std::{collections::BTreeMap, time::Instant};
use bevy_ecs::{component::{ComponentHooks, StorageType}, prelude::*};
use bevy_stardust_quic::{RecvStreamId, SendStreamId};
use quinn_proto::{ConnectionEvent, ConnectionHandle as QuinnHandle, EndpointEvent, Event as ApplicationEvent, StreamId as QuinnStreamId};
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
    map_qsid_ssid: BTreeMap<QuinnStreamId, SendStreamId>,
    map_rsid_qsid: BTreeMap<RecvStreamId, QuinnStreamId>,
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
            map_qsid_ssid: BTreeMap::new(),
            map_rsid_qsid: BTreeMap::new(),
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

    pub fn handle_timeouts(
        &mut self,
        now: Instant,
    ) {
        self.connection.handle_timeout(now);
        self.statemachine.handle_timeout(now);
    }

    #[inline]
    pub fn handle_connection_event(
        &mut self,
        event: ConnectionEvent
    ) {
        self.connection.handle_event(event);
    }

    #[inline]
    pub fn poll_endpoint_events(
        &mut self,
    ) -> Option<EndpointEvent> {
        self.connection.poll_endpoint_events()
    }
}

#[inline]
pub(crate) fn qsid_to_rsid(id: QuinnStreamId) -> RecvStreamId {
    RecvStreamId(id.0)
}

#[inline]
pub(crate) fn qsid_to_ssid(id: QuinnStreamId) -> SendStreamId {
    SendStreamId(id.0)
}

#[inline]
pub(crate) fn rsid_to_qsid(id: RecvStreamId) -> QuinnStreamId {
    QuinnStreamId(id.0)
}

#[inline]
pub(crate) fn ssid_to_qsid(id: SendStreamId) -> QuinnStreamId {
    QuinnStreamId(id.0)
}