use std::{collections::BTreeMap, sync::Arc};
use bevy_ecs::{component::{ComponentHooks, StorageType}, prelude::*};
use quinn_proto::{ConnectionHandle as QuinnHandle, EndpointConfig, EndpointEvent, ServerConfig};
use crate::socket::QuicSocket;

/// A QUIC endpoint.
pub struct Endpoint(pub(crate) Box<EndpointInner>);

impl Component for Endpoint {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_remove(|mut world, entity, _| {
            todo!()
        });
    }
}

pub(crate) struct EndpointInner {
    socket: QuicSocket,

    endpoint: quinn_proto::Endpoint,

    connections: EndpointConnections,
}

impl EndpointInner {
    pub fn new(
        socket: QuicSocket,
        config: Arc<EndpointConfig>,
        server: Option<Arc<ServerConfig>>,
    ) -> Self {
        Self {
            socket,

            endpoint: quinn_proto::Endpoint::new(
                config,
                server,
                true,
                None,
            ),

            connections: EndpointConnections::new(),
        }
    }

    pub unsafe fn inform_connection_close(
        &mut self,
        entity: Entity,
    ) {
        // Remove the handle from the connection map.
        let handle = match self.connections.remove_by_entity(entity) {
            Some(handle) => handle,
            None => return,
        };

        // Inform the Quinn state machine that the endpoint has been removed.
        self.endpoint.handle_event(handle, EndpointEvent::drained());
    }
}

struct EndpointConnections {
    e2h: BTreeMap<Entity, QuinnHandle>,
    h2e: BTreeMap<QuinnHandle, Entity>,
}

impl EndpointConnections {
    fn new() -> Self {
        Self {
            e2h: BTreeMap::new(),
            h2e: BTreeMap::new(),
        }
    }

    unsafe fn remove_by_entity(&mut self, entity: Entity) -> Option<QuinnHandle> {
        let handle = self.e2h.remove(&entity)?;
        self.h2e.remove(&handle);
        return Some(handle);
    }

    unsafe fn remove_by_handle(&mut self, handle: QuinnHandle) -> Option<Entity> {
        let entity = self.h2e.remove(&handle)?;
        self.e2h.remove(&entity);
        return Some(entity);
    }
}